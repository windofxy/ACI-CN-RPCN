use crate::server::public_api::PublicApiSharedState;
use crate::server::public_api::api_state::handle_api_state_request;
use crate::server::public_api::r#const::{MAX_REQUEST_ID_LENGTH, MESSAGE_TYPE_API_STATE_REQUEST, MESSAGE_TYPE_ERROR_RESPONSE, MESSAGE_TYPE_GET_ROOM_LIST_REQUEST, PUBLIC_API_NAME, PUBLIC_API_VERSION};
use crate::server::public_api::envelope::{PublicApiRequestEnvelope, PublicApiResponseEnvelope};
use crate::server::public_api::error::PublicApiError;
use crate::server::public_api::get_room_list::handle_get_room_list_request;

pub fn route_request_text(request_text: &str, shared_state: &PublicApiSharedState) -> Result<Option<String>, serde_json::Error> {
	let request_parse_result: Result<PublicApiRequestEnvelope, PublicApiError> = serde_json::from_str(request_text).map_err(PublicApiError::from);
	let request = match request_parse_result {
		Ok(request) => request,
		Err(error) => {
			let response = PublicApiResponseEnvelope::<serde_json::Value>::error(MESSAGE_TYPE_ERROR_RESPONSE, None, error.to_error_info());
			return response.to_json_string().map(Some);
		}
	};

	if request.api_name != PUBLIC_API_NAME {
		return Ok(None);
	}

	if request.api_version != PUBLIC_API_VERSION {
		let response = PublicApiResponseEnvelope::<serde_json::Value>::error(MESSAGE_TYPE_ERROR_RESPONSE, request.request_id, PublicApiError::InvalidApiVersion(request.api_version).to_error_info());
		return response.to_json_string().map(Some);
	}

	if request.request_id.as_ref().is_some_and(|request_id| request_id.len() > MAX_REQUEST_ID_LENGTH) {
		let response = PublicApiResponseEnvelope::<serde_json::Value>::error(MESSAGE_TYPE_ERROR_RESPONSE, request.request_id, PublicApiError::InvalidRequestId.to_error_info());
		return response.to_json_string().map(Some);
	}

	let request_id = request.request_id.clone();

	let response =
		match request.message_type.as_str() {
			MESSAGE_TYPE_API_STATE_REQUEST => handle_api_state_request(request)
				.or_else(|error| PublicApiResponseEnvelope::<serde_json::Value>::error(MESSAGE_TYPE_ERROR_RESPONSE, request_id, error.to_error_info()).to_json_string()),
			MESSAGE_TYPE_GET_ROOM_LIST_REQUEST => handle_get_room_list_request(request, shared_state)
				.or_else(|error| PublicApiResponseEnvelope::<serde_json::Value>::error(MESSAGE_TYPE_ERROR_RESPONSE, request_id, error.to_error_info()).to_json_string()),
			other => PublicApiResponseEnvelope::<serde_json::Value>::error(MESSAGE_TYPE_ERROR_RESPONSE, request_id, PublicApiError::UnsupportedMessageType(other.to_string()).to_error_info())
				.to_json_string(),
		}?;

	Ok(Some(response))
}
