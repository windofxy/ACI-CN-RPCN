use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::server::client::Client;
use crate::server::public_api::r#const::{PUBLIC_API_NAME, PUBLIC_API_VERSION};
use crate::server::public_api::error::PublicApiErrorInfo;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicApiRequestEnvelope {
	pub api_name: String,
	pub api_version: String,
	#[serde(default)]
	pub request_id: Option<String>,
	pub message_type: String,
	#[serde(default)]
	pub data: Option<Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicApiResponseEnvelope<T>
where
	T: Serialize,
{
	pub api_name: &'static str,
	pub api_version: &'static str,
	pub timestamp: u64,
	pub message_type: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub request_id: Option<String>,
	pub data: Option<T>,
	pub error: Option<PublicApiErrorInfo>,
}

impl<T> PublicApiResponseEnvelope<T>
where
	T: Serialize,
{
	pub fn success(message_type: impl Into<String>, request_id: Option<String>, data: T) -> PublicApiResponseEnvelope<T> {
		PublicApiResponseEnvelope {
			api_name: PUBLIC_API_NAME,
			api_version: PUBLIC_API_VERSION,
			timestamp: Client::get_timestamp_nanos() / 1_000_000,
			message_type: message_type.into(),
			request_id,
			data: Some(data),
			error: None,
		}
	}

	pub fn error(message_type: impl Into<String>, request_id: Option<String>, error: PublicApiErrorInfo) -> PublicApiResponseEnvelope<T> {
		PublicApiResponseEnvelope {
			api_name: PUBLIC_API_NAME,
			api_version: PUBLIC_API_VERSION,
			timestamp: Client::get_timestamp_nanos() / 1_000_000,
			message_type: message_type.into(),
			request_id,
			data: None,
			error: Some(error),
		}
	}

	pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
		serde_json::to_string(self)
	}
}
