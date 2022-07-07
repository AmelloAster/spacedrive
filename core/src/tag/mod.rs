use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

use crate::{
	file::File,
	prisma::{
		self, file,
		tag::{self},
		tag_on_file,
	},
	CoreContext, CoreError, CoreResponse,
};

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Tag {
	pub id: i32,
	pub pub_id: String,
	pub name: Option<String>,
	pub color: Option<String>,

	pub total_files: Option<i32>,
	pub redundancy_goal: Option<i32>,

	pub date_created: chrono::DateTime<chrono::Utc>,
	pub date_modified: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TagOnFile {
	pub tag_id: i32,
	pub tag: Option<Tag>,

	pub file_id: i32,
	pub file: Option<File>,

	pub date_created: chrono::DateTime<chrono::Utc>,
}

impl Into<Tag> for tag::Data {
	fn into(self) -> Tag {
		Tag {
			id: self.id,
			pub_id: self.pub_id,
			name: self.name,
			color: self.color,
			total_files: self.total_files,
			redundancy_goal: self.redundancy_goal,
			date_created: self.date_created.into(),
			date_modified: self.date_modified.into(),
		}
	}
}

impl Into<TagOnFile> for tag_on_file::Data {
	fn into(self) -> TagOnFile {
		TagOnFile {
			tag_id: self.tag_id,
			tag: self.tag.map(|t| (*t).into()),
			file_id: self.file_id,
			file: self.file.map(|f| (*f).into()),
			date_created: self.date_created.into(),
		}
	}
}

#[derive(Serialize, Deserialize, TS, Debug)]
#[ts(export)]
pub struct TagWithFiles {
	pub tag: Tag,
	pub files_with_tag: Vec<TagOnFile>,
}

#[derive(Error, Debug)]
pub enum TagError {
	#[error("Tag not found")]
	TagNotFound(i32),
	#[error("Database error")]
	DatabaseError(#[from] prisma::QueryError),
}

pub async fn create_tag(
	ctx: CoreContext,
	name: String,
	color: String,
) -> Result<CoreResponse, CoreError> {
	let created_tag = ctx
		.database
		.tag()
		.create(
			tag::pub_id::set(uuid::Uuid::new_v4().to_string()),
			vec![tag::name::set(Some(name)), tag::color::set(Some(color))],
		)
		.exec()
		.await
		.unwrap();

	Ok(CoreResponse::TagCreateResponse {
		pub_id: created_tag.pub_id,
	})
}

pub async fn update_tag(
	ctx: CoreContext,
	id: i32,
	name: Option<String>,
	color: Option<String>,
) -> Result<CoreResponse, CoreError> {
	ctx.database
		.tag()
		.find_unique(tag::id::equals(id))
		.update(vec![tag::name::set(name), tag::color::set(color)])
		.exec()
		.await
		.unwrap();

	Ok(CoreResponse::Success(()))
}

pub async fn tag_unassign(
	ctx: CoreContext,
	file_id: i32,
	tag_id: i32,
) -> Result<CoreResponse, CoreError> {
	ctx.database
		.tag_on_file()
		.find_unique(tag_on_file::tag_id_file_id(tag_id, file_id))
		.delete()
		.exec()
		.await
		.unwrap();

	Ok(CoreResponse::Success(()))
}

pub async fn tag_assign(
	ctx: CoreContext,
	file_id: i32,
	tag_id: i32,
) -> Result<CoreResponse, CoreError> {
	ctx.database
		.tag_on_file()
		.create(
			tag_on_file::tag::link(tag::id::equals(tag_id)),
			tag_on_file::file::link(file::id::equals(file_id)),
			vec![],
		)
		.exec()
		.await
		.unwrap();

	Ok(CoreResponse::Success(()))
}

pub async fn tag_delete(ctx: CoreContext, id: i32) -> Result<CoreResponse, CoreError> {
	ctx.database
		.tag()
		.find_unique(tag::id::equals(id))
		.delete()
		.exec()
		.await?
		.unwrap();

	Ok(CoreResponse::Success(()))
}

pub async fn get_tag(ctx: CoreContext, id: i32) -> Result<CoreResponse, CoreError> {
	let tag = ctx
		.database
		.tag()
		.find_unique(tag::id::equals(id))
		.exec()
		.await?
		.map(Into::into);

	Ok(CoreResponse::GetTag(tag))
}

pub async fn get_all_tags(
	ctx: CoreContext,
	name_starts_with: Option<String>,
) -> Result<CoreResponse, CoreError> {
	let tags: Vec<Tag> = ctx
		.database
		.tag()
		.find_many(vec![tag::name::starts_with(
			name_starts_with.unwrap_or(String::new()),
		)])
		.exec()
		.await?
		.into_iter()
		.map(Into::into)
		.collect();

	Ok(CoreResponse::GetTags(tags))
}
