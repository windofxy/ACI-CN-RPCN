use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::server::client::{COMMUNICATION_ID_SIZE, ComId, com_id_to_string};
use crate::server::public_api::PublicApiSharedState;
use crate::server::public_api::r#const::MESSAGE_TYPE_GET_ROOM_LIST_RESPONSE;
use crate::server::public_api::envelope::{PublicApiRequestEnvelope, PublicApiResponseEnvelope};
use crate::server::public_api::error::PublicApiError;
use crate::server::room_manager::RoomSnapshot;
use crate::server::stream_extractor::np2_structs::{
	BinAttr, BinAttrInternal, IntAttr, RoomDataExternal, RoomDataInternal, RoomGroup, RoomMemberBinAttrInternal, RoomMemberDataInternal, Uint8, Uint16, UserInfo,
};

#[derive(Debug, Clone, Default, Deserialize)]
pub struct GetRoomListRequestData {
	#[serde(rename = "communicationID")]
	pub communication_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(transparent)]
pub struct GetRoomListResponseData {
	pub rooms_by_communication_id: BTreeMap<String, GetRoomListCommunicationData>,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetRoomListCommunicationData {
	pub player_count: i64,
	pub room_list: Vec<GetRoomListRoomEntry>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetRoomListRoomEntry {
	pub current_player_count: u32,
	pub max_player_count: u32,
	pub host_player_name: String,
	pub room_info: GetRoomListRoomInfo,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetRoomListRoomInfo {
	pub room_data_external: Value,
	pub room_data_internal: Value,
}

pub fn handle_get_room_list_request(request: PublicApiRequestEnvelope, shared_state: &PublicApiSharedState) -> Result<String, PublicApiError> {
	let request_id = request.request_id.clone();
	let request_data_result: Result<GetRoomListRequestData, PublicApiError> = match request.data {
		Some(data) => serde_json::from_value(data).map_err(PublicApiError::from),
		None => Ok(GetRoomListRequestData::default()),
	};

	let request_data = match request_data_result {
		Ok(request_data) => request_data,
		Err(error) => {
			let response = PublicApiResponseEnvelope::<GetRoomListResponseData>::error(MESSAGE_TYPE_GET_ROOM_LIST_RESPONSE, request_id, error.to_error_info());
			return response.to_json_string().map_err(PublicApiError::from);
		}
	};

	let communication_id = match request_data.communication_id.as_deref() {
		Some(communication_id) => match parse_communication_id(communication_id) {
			Ok(communication_id) => Some(communication_id),
			Err(error) => {
				let response = PublicApiResponseEnvelope::<GetRoomListResponseData>::error(MESSAGE_TYPE_GET_ROOM_LIST_RESPONSE, request_id, error.to_error_info());
				return response.to_json_string().map_err(PublicApiError::from);
			}
		},
		None => None,
	};

	let room_snapshots = shared_state.room_manager.read().list_room_snapshots(communication_id.as_ref());
	let player_counts = collect_player_counts(shared_state);
	let response_data = build_response_data(communication_id.as_ref(), &room_snapshots, &player_counts);
	let response = PublicApiResponseEnvelope::success(MESSAGE_TYPE_GET_ROOM_LIST_RESPONSE, request_id, response_data);

	response.to_json_string().map_err(PublicApiError::from)
}

fn parse_communication_id(communication_id: &str) -> Result<ComId, PublicApiError> {
	let bytes = communication_id.as_bytes();
	if bytes.len() != COMMUNICATION_ID_SIZE {
		return Err(PublicApiError::InvalidCommunicationId(communication_id.to_string()));
	}

	let mut com_id = [0u8; COMMUNICATION_ID_SIZE];
	com_id.copy_from_slice(bytes);
	Ok(com_id)
}

fn collect_player_counts(shared_state: &PublicApiSharedState) -> BTreeMap<String, i64> {
	shared_state
		.game_tracker
		.psn_games
		.read()
		.iter()
		.map(|(com_id, game_info)| (com_id_to_string(com_id), game_info.num_users.load(std::sync::atomic::Ordering::SeqCst)))
		.collect()
}

fn build_response_data(communication_id: Option<&ComId>, room_snapshots: &[RoomSnapshot], player_counts: &BTreeMap<String, i64>) -> GetRoomListResponseData {
	let mut rooms_by_communication_id: BTreeMap<String, GetRoomListCommunicationData> = BTreeMap::new();

	if let Some(communication_id) = communication_id {
		let communication_id_string = com_id_to_string(communication_id);
		rooms_by_communication_id.insert(
			communication_id_string.clone(),
			GetRoomListCommunicationData {
				player_count: *player_counts.get(&communication_id_string).unwrap_or(&0),
				room_list: Vec::new(),
			},
		);
	} else {
		for (communication_id, player_count) in player_counts {
			rooms_by_communication_id.insert(
				communication_id.clone(),
				GetRoomListCommunicationData {
					player_count: *player_count,
					room_list: Vec::new(),
				},
			);
		}
	}

	for room_snapshot in room_snapshots {
		let communication_id = com_id_to_string(&room_snapshot.communication_id);
		let host_player_name = room_snapshot.room_data_external.owner.as_ref().map(|owner| owner.online_name.clone()).unwrap_or_default();
		let room_entry = GetRoomListRoomEntry {
			current_player_count: room_snapshot.current_player_count,
			max_player_count: room_snapshot.max_player_count,
			host_player_name,
			room_info: GetRoomListRoomInfo {
				room_data_external: room_data_external_to_json(&room_snapshot.room_data_external),
				room_data_internal: room_data_internal_to_json(&room_snapshot.room_data_internal),
			},
		};

		rooms_by_communication_id
			.entry(communication_id)
			.or_insert_with(|| GetRoomListCommunicationData {
				player_count: 0,
				room_list: Vec::new(),
			})
			.room_list
			.push(room_entry);
	}

	GetRoomListResponseData { rooms_by_communication_id }
}

fn uint16_value(value: &Option<Uint16>) -> u16 {
	value.as_ref().map(|value| value.value as u16).unwrap_or_default()
}

fn uint8_value(value: &Option<Uint8>) -> u8 {
	value.as_ref().map(|value| value.value as u8).unwrap_or_default()
}

fn user_info_to_json(user_info: &UserInfo) -> Value {
	json!({
		"npId": user_info.np_id,
		"onlineName": user_info.online_name,
		"avatarUrl": user_info.avatar_url,
	})
}

fn room_group_to_json(room_group: &RoomGroup) -> Value {
	json!({
		"groupId": uint8_value(&room_group.group_id),
		"withPassword": room_group.with_password,
		"label": room_group.label,
		"slotNum": room_group.slot_num,
		"curGroupMemberNum": room_group.cur_group_member_num,
	})
}

fn int_attr_to_json(int_attr: &IntAttr) -> Value {
	json!({
		"id": uint16_value(&int_attr.id),
		"num": int_attr.num,
	})
}

fn bin_attr_to_json(bin_attr: &BinAttr) -> Value {
	json!({
		"id": uint16_value(&bin_attr.id),
		"data": bin_attr.data,
	})
}

fn room_member_bin_attr_internal_to_json(bin_attr: &RoomMemberBinAttrInternal) -> Value {
	json!({
		"updateDate": bin_attr.update_date,
		"data": bin_attr.data.as_ref().map(bin_attr_to_json),
	})
}

fn bin_attr_internal_to_json(bin_attr: &BinAttrInternal) -> Value {
	json!({
		"updateDate": bin_attr.update_date,
		"updateMemberId": uint16_value(&bin_attr.update_member_id),
		"data": bin_attr.data.as_ref().map(bin_attr_to_json),
	})
}

fn room_member_data_internal_to_json(member: &RoomMemberDataInternal) -> Value {
	json!({
		"userInfo": member.user_info.as_ref().map(user_info_to_json),
		"joinDate": member.join_date,
		"memberId": member.member_id,
		"teamId": uint8_value(&member.team_id),
		"roomGroup": member.room_group.as_ref().map(room_group_to_json),
		"natType": uint8_value(&member.nat_type),
		"flagAttr": member.flag_attr,
		"roomMemberBinAttrInternal": member.room_member_bin_attr_internal.iter().map(room_member_bin_attr_internal_to_json).collect::<Vec<_>>(),
	})
}

fn room_data_internal_to_json(room_data: &RoomDataInternal) -> Value {
	json!({
		"serverId": uint16_value(&room_data.server_id),
		"worldId": room_data.world_id,
		"lobbyId": room_data.lobby_id,
		"roomId": room_data.room_id,
		"passwordSlotMask": room_data.password_slot_mask,
		"maxSlot": room_data.max_slot,
		"memberList": room_data.member_list.iter().map(room_member_data_internal_to_json).collect::<Vec<_>>(),
		"ownerId": uint16_value(&room_data.owner_id),
		"roomGroup": room_data.room_group.iter().map(room_group_to_json).collect::<Vec<_>>(),
		"flagAttr": room_data.flag_attr,
		"roomBinAttrInternal": room_data.room_bin_attr_internal.iter().map(bin_attr_internal_to_json).collect::<Vec<_>>(),
	})
}

fn room_data_external_to_json(room_data: &RoomDataExternal) -> Value {
	json!({
		"serverId": uint16_value(&room_data.server_id),
		"worldId": room_data.world_id,
		"publicSlotNum": uint16_value(&room_data.public_slot_num),
		"privateSlotNum": uint16_value(&room_data.private_slot_num),
		"lobbyId": room_data.lobby_id,
		"roomId": room_data.room_id,
		"openPublicSlotNum": uint16_value(&room_data.open_public_slot_num),
		"maxSlot": uint16_value(&room_data.max_slot),
		"openPrivateSlotNum": uint16_value(&room_data.open_private_slot_num),
		"curMemberNum": uint16_value(&room_data.cur_member_num),
		"passwordSlotMask": room_data.password_slot_mask,
		"owner": room_data.owner.as_ref().map(user_info_to_json),
		"roomGroup": room_data.room_group.iter().map(room_group_to_json).collect::<Vec<_>>(),
		"flagAttr": room_data.flag_attr,
		"roomSearchableIntAttrExternal": room_data.room_searchable_int_attr_external.iter().map(int_attr_to_json).collect::<Vec<_>>(),
		"roomSearchableBinAttrExternal": room_data.room_searchable_bin_attr_external.iter().map(bin_attr_to_json).collect::<Vec<_>>(),
		"roomBinAttrExternal": room_data.room_bin_attr_external.iter().map(bin_attr_to_json).collect::<Vec<_>>(),
	})
}
