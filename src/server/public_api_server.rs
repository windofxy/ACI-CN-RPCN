use std::io;
use std::net::ToSocketAddrs;

use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use tracing::{info, warn};

use crate::server::Server;
use crate::server::client::TerminateWatch;
use crate::server::public_api::PublicApiSharedState;
use crate::server::public_api::error::PublicApiError;
use crate::server::public_api::router::route_request_text;

pub struct PublicApiServer {
	listener: TcpListener,
	term_watch: TerminateWatch,
	shared_state: PublicApiSharedState,
}

impl Server {
	pub async fn start_public_api_server(&self, term_watch: TerminateWatch) -> io::Result<()> {
		let bind_addr = self.config.read().get_public_api_binds().clone();

		if let Some((host, port)) = bind_addr {
			let str_addr = format!("{}:{}", host, port);
			let mut addr = str_addr
				.to_socket_addrs()
				.map_err(|e| io::Error::new(e.kind(), format!("Public API: {} is not a valid address", &str_addr)))?;
			let addr = addr
				.next()
				.ok_or_else(|| io::Error::new(io::ErrorKind::AddrNotAvailable, format!("Public API: {} is not a valid address", &str_addr)))?;

			let listener = TcpListener::bind(addr)
				.await
				.map_err(|e| io::Error::new(e.kind(), format!("Public API: error binding to <{}>: {}", &addr, e)))?;

			info!("Public API server now waiting for connections on {}", str_addr);

			let shared_state = PublicApiSharedState::new(self.room_manager.clone(), self.game_tracker.clone());
			let mut public_api_server = PublicApiServer::new(listener, term_watch, shared_state);
			tokio::task::spawn(async move {
				public_api_server.server_proc().await;
			});
		}

		Ok(())
	}
}

impl PublicApiServer {
	fn new(listener: TcpListener, term_watch: TerminateWatch, shared_state: PublicApiSharedState) -> PublicApiServer {
		PublicApiServer { listener, term_watch, shared_state }
	}

	async fn server_proc(&mut self) {
		if *self.term_watch.recv.borrow_and_update() {
			return;
		}

		'public_api_loop: loop {
			tokio::select! {
				accept_res = self.listener.accept() => {
					match accept_res {
						Err(e) => {
							warn!("Public API: error accepting a client: {}", e);
							continue 'public_api_loop;
						}
						Ok((stream, peer_addr)) => {
							info!("Public API: new client from {}", peer_addr);
							let shared_state = self.shared_state.clone();
							tokio::spawn(async move {
								if let Err(e) = PublicApiServer::handle_connection(stream, shared_state).await {
									warn!("Public API: connection terminated with error: {}", e);
								}
							});
						}
					}
				}
				_ = self.term_watch.recv.changed() => {
					break 'public_api_loop;
				}
			}
		}

		info!("Public API server shutting down");
	}

	async fn handle_connection(stream: TcpStream, shared_state: PublicApiSharedState) -> Result<(), String> {
		let mut websocket = accept_async(stream).await.map_err(|e| format!("WebSocket handshake failed: {}", e))?;

		while let Some(message) = websocket.next().await {
			let message = message.map_err(|e| format!("WebSocket receive failed: {}", e))?;

			match message {
				Message::Text(text) => {
					if let Some(response_text) = route_request_text(text.as_str(), &shared_state).map_err(|e| format!("Public API request failed: {}", e))? {
						websocket.send(Message::Text(response_text.into())).await.map_err(|e| format!("WebSocket send failed: {}", e))?;
					}
				}
				Message::Binary(data) => {
					let text = String::from_utf8(data.to_vec()).map_err(|_| PublicApiError::InvalidUtf8.to_string())?;
					if let Some(response_text) = route_request_text(&text, &shared_state).map_err(|e| format!("Public API request failed: {}", e))? {
						websocket.send(Message::Text(response_text.into())).await.map_err(|e| format!("WebSocket send failed: {}", e))?;
					}
				}
				Message::Ping(payload) => {
					websocket.send(Message::Pong(payload)).await.map_err(|e| format!("WebSocket pong failed: {}", e))?;
				}
				Message::Close(_) => break,
				Message::Pong(_) => {}
				Message::Frame(_) => {}
			}
		}

		Ok(())
	}
}
