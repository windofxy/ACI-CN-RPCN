use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::server::public_api::r#const::{MESSAGE_TYPE_API_STATE_RESPONSE, PUBLIC_API_VERSION};
use crate::server::public_api::envelope::{PublicApiRequestEnvelope, PublicApiResponseEnvelope};
use crate::server::public_api::error::PublicApiError;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ApiStateRequestData {}

#[derive(Debug, Clone, Serialize)]
pub struct ApiStateResponseData {
	#[serde(rename = "serverAPIVersion")]
	pub server_api_version: &'static str,
}

impl ApiStateResponseData {
	pub fn new() -> ApiStateResponseData {
		ApiStateResponseData {
			server_api_version: PUBLIC_API_VERSION,
		}
	}
}

pub fn handle_api_state_request(request: PublicApiRequestEnvelope) -> Result<String, PublicApiError> {
	let request_id = request.request_id.clone();
	let request_data_result: Result<ApiStateRequestData, PublicApiError> = match request.data {
		Some(data) => serde_json::from_value(data).map_err(PublicApiError::from),
		None => serde_json::from_value(Value::Object(Default::default())).map_err(PublicApiError::from),
	};

	if let Err(error) = request_data_result {
		let response = PublicApiResponseEnvelope::<ApiStateResponseData>::error(MESSAGE_TYPE_API_STATE_RESPONSE, request_id, error.to_error_info());
		return response.to_json_string().map_err(PublicApiError::from);
	}

	let response = PublicApiResponseEnvelope::success(MESSAGE_TYPE_API_STATE_RESPONSE, request_id, ApiStateResponseData::new());

	response.to_json_string().map_err(PublicApiError::from)
}
