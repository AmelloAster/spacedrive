use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// An operation on a shared record CRDT.
/// Shared records are identified by their `model` (db table) and `id` (uuid).
///
/// ## Create
/// Creating a shared record simply requires providing its properties.
/// The record is created with the type of `model` and the provided `id`,
/// along with the provided data.
///
/// ## Update
/// Updates to shared records must be done on a per-field basis,
/// ie. multiple fields cannot be updated in a single operation.
/// If multiple updates were permitted in one operation, determining the most
/// recent update for a field would be significantly more difficult,
/// since each operation would have to be searched for what fields they affect.
/// Sure, it could be done, but requiring one operation per update is more simple.
///
/// ## Delete
/// Deleting a shared record uses the operation's `record_id` and `model` to identify the record and delete.
#[derive(Serialize, Deserialize, Clone)]
pub struct SharedOperation {
	#[serde(rename = "r")]
	pub record_id: Value, // Uuid,
	#[serde(rename = "m")]
	pub model: String,
	#[serde(rename = "d")]
	pub data: SharedOperationData,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum SharedOperationData {
	#[serde(rename = "c")]
	Create(SharedOperationCreateData),
	#[serde(rename = "u")]
	Update {
		#[serde(rename = "f")]
		field: String,
		#[serde(rename = "v")]
		value: Value,
	},
	#[serde(rename = "d")]
	Delete,
}

impl SharedOperationData {
	pub fn create_unique(data: Map<String, Value>) -> Self {
		Self::Create(SharedOperationCreateData::Unique(data))
	}

	pub fn create_atomic() -> Self {
		Self::Create(SharedOperationCreateData::Atomic)
	}

	pub fn update(field: String, value: Value) -> Self {
		Self::Update { field, value }
	}

	pub fn delete() -> Self {
		Self::Delete
	}
}

#[derive(Serialize, Deserialize, Clone)]
pub enum SharedOperationCreateData {
	#[serde(rename = "u")]
	Unique(Map<String, Value>),
	#[serde(rename = "a")]
	Atomic,
}
