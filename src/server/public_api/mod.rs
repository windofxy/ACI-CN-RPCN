use std::sync::Arc;

use parking_lot::RwLock;

use crate::server::game_tracker::GameTracker;
use crate::server::room_manager::RoomManager;

pub mod api_state;
pub mod r#const;
pub mod envelope;
pub mod error;
pub mod get_room_list;
pub mod router;

#[derive(Clone)]
pub struct PublicApiSharedState {
	pub room_manager: Arc<RwLock<RoomManager>>,
	pub game_tracker: Arc<GameTracker>,
}

impl PublicApiSharedState {
	pub fn new(room_manager: Arc<RwLock<RoomManager>>, game_tracker: Arc<GameTracker>) -> PublicApiSharedState {
		PublicApiSharedState { room_manager, game_tracker }
	}
}
