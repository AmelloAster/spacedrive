use chrono::{DateTime, Utc};
use int_enum::IntEnum;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
mod config;
use crate::prisma::node;
pub use config::*;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LibraryNode {
	pub uuid: String,
	pub name: String,
	pub platform: Platform,
	pub last_seen: DateTime<Utc>,
}

impl From<node::Data> for LibraryNode {
	fn from(data: node::Data) -> Self {
		Self {
			uuid: data.pub_id,
			name: data.name,
			platform: IntEnum::from_int(data.platform).unwrap(),
			last_seen: data.last_seen.into(),
		}
	}
}

impl From<Box<node::Data>> for LibraryNode {
	fn from(data: Box<node::Data>) -> Self {
		Self::from(*data)
	}
}

#[allow(clippy::upper_case_acronyms)]
#[repr(i32)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS, Eq, PartialEq, IntEnum)]
#[ts(export)]
pub enum Platform {
	Unknown = 0,
	Windows = 1,
	MacOS = 2,
	Linux = 3,
	IOS = 4,
	Android = 5,
}
