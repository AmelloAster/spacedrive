use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Serialize, Deserialize, Clone)]
pub struct OwnedOperation {
	#[serde(rename = "m")]
	pub model: String,
	#[serde(rename = "d")]
	pub data: Vec<OwnedOperationData>,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum OwnedOperationData {
	#[serde(rename = "c")]
	Create(Map<String, Value>),
	#[serde(rename = "u")]
	Update(Map<String, Value>),
	#[serde(rename = "d")]
	Delete(Value),
}
