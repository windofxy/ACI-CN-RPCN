use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicApiErrorInfo {
	pub code: &'static str,
	pub message: String,
}

#[derive(Debug, Error)]
pub enum PublicApiError {
	#[error("WebSocket payload is not valid UTF-8 text")]
	InvalidUtf8,
	#[error("Failed to parse JSON request: {0}")]
	InvalidJson(#[from] serde_json::Error),
	#[error("Unsupported apiVersion: {0}")]
	InvalidApiVersion(String),
	#[error("Unsupported message type: {0}")]
	UnsupportedMessageType(String),
	#[error("requestID is longer than the supported limit")]
	InvalidRequestId,
	#[error("Invalid communicationID: {0}")]
	InvalidCommunicationId(String),
}

impl PublicApiError {
	pub fn code(&self) -> &'static str {
		match self {
			PublicApiError::InvalidUtf8 => "INVALID_UTF8",
			PublicApiError::InvalidJson(_) => "INVALID_JSON",
			PublicApiError::InvalidApiVersion(_) => "INVALID_API_VERSION",
			PublicApiError::UnsupportedMessageType(_) => "UNSUPPORTED_MESSAGE_TYPE",
			PublicApiError::InvalidRequestId => "INVALID_REQUEST_ID",
			PublicApiError::InvalidCommunicationId(_) => "INVALID_COMMUNICATION_ID",
		}
	}

	pub fn to_error_info(&self) -> PublicApiErrorInfo {
		PublicApiErrorInfo {
			code: self.code(),
			message: self.to_string(),
		}
	}
}
