use std::convert::Infallible;
use std::io;
use std::path::PathBuf;
use std::pin::Pin;
use std::task::{Context, Poll};

use hyper::body::{Body, Bytes, Frame};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use tokio::fs;
use tokio::net::TcpListener;
use tracing::{info, warn};

use crate::server::Server;
use crate::server::client::TerminateWatch;

const TSS_DATA_DIR: &str = "tss_data";

struct TssBody(Option<Vec<u8>>);

impl TssBody {
	fn new(data: Vec<u8>) -> Self {
		TssBody(Some(data))
	}

	fn empty() -> Self {
		TssBody(None)
	}
}

impl Body for TssBody {
	type Data = Bytes;
	type Error = Infallible;

	fn poll_frame(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Result<Frame<Bytes>, Infallible>>> {
		match self.0.take() {
			Some(data) if !data.is_empty() => Poll::Ready(Some(Ok(Frame::data(Bytes::from(data))))),
			_ => Poll::Ready(None),
		}
	}
}

impl Server {
	pub async fn start_tss_server(&self, term_watch: TerminateWatch) -> io::Result<()> {
		let host = self.config.read().get_host_ipv4().clone();
		let tss_port = self
			.config
			.read()
			.get_port()
			.parse::<u16>()
			.unwrap_or(31313)
			.saturating_add(2);

		let addr = format!("{}:{}", host, tss_port);

		let listener = TcpListener::bind(&addr).await.map_err(|e| io::Error::new(e.kind(), format!("TSS server failed to bind to <{}>: {}", &addr, e)))?;

		info!("TSS HTTP server listening on <{}>", &addr);

		tokio::spawn(async move {
			let mut term = term_watch;
			loop {
				tokio::select! {
					accept_res = listener.accept() => {
						match accept_res {
							Err(e) => {
								warn!("TSS server accept failed: {}", e);
								continue;
							}
							Ok((stream, _peer)) => {
								let io = TokioIo::new(stream);
								tokio::spawn(async move {
									if let Err(e) = http1::Builder::new()
										.keep_alive(false)
										.serve_connection(io, service_fn(handle_tss_request))
										.await
									{
										warn!("TSS: error serving connection: {}", e);
									}
								});
							}
						}
					}
					_ = term.recv.changed() => break,
				}
			}
			info!("TSS server shutting down");
		});

		Ok(())
	}
}

async fn handle_tss_request(req: Request<hyper::body::Incoming>) -> Result<Response<TssBody>, Infallible> {
	if req.method() != Method::GET {
		return Ok(Response::builder().status(StatusCode::METHOD_NOT_ALLOWED).body(TssBody::empty()).unwrap());
	}

	let path = req.uri().path().to_owned();
	// Expected path: /tss/<com_id>/<filename>
	let rel = path.trim_start_matches('/');
	let parts: Vec<&str> = rel.splitn(3, '/').collect();

	if parts.len() != 3 || parts[0] != "tss" {
		return Ok(Response::builder().status(StatusCode::NOT_FOUND).body(TssBody::empty()).unwrap());
	}

	let com_id = parts[1];
	let filename = parts[2];

	// Reject path traversal attempts
	if com_id.contains("..") || com_id.contains('/') || com_id.contains('\\') || filename.contains("..") || filename.contains('/') || filename.contains('\\') {
		return Ok(Response::builder().status(StatusCode::BAD_REQUEST).body(TssBody::empty()).unwrap());
	}

	let file_path = PathBuf::from(TSS_DATA_DIR).join(com_id).join(filename);

	match fs::read(&file_path).await {
		Ok(data) => Ok(Response::builder()
			.status(StatusCode::OK)
			.header("Content-Type", "application/octet-stream")
			.body(TssBody::new(data))
			.unwrap()),
		Err(_) => Ok(Response::builder().status(StatusCode::NOT_FOUND).body(TssBody::empty()).unwrap()),
	}
}
