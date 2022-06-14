use crate::{
	album::{Album, AlbumError, AlbumWithFiles, FileInAlbum},
	encode::THUMBNAIL_CACHE_DIR_NAME,
	file::{DirectoryWithContents, FileError, FilePath},
	node::get_nodestate,
	prisma::{album, file_in_album, file_path},
	sys::get_location,
	CoreContext,
};
use std::path::Path;

pub async fn open_dir(
	ctx: &CoreContext,
	location_id: &i32,
	path: &str,
) -> Result<DirectoryWithContents, FileError> {
	let db = &ctx.database;
	let config = get_nodestate();

	// get location
	let location = get_location(ctx, location_id.clone()).await?;

	let directory = db
		.file_path()
		.find_first(vec![
			file_path::location_id::equals(location.id),
			file_path::materialized_path::equals(path.into()),
			file_path::is_dir::equals(true),
		])
		.exec()
		.await?
		.ok_or(FileError::DirectoryNotFound(path.to_string()))?;

	println!("DIRECTORY: {:?}", directory);

	let mut file_paths: Vec<FilePath> = db
		.file_path()
		.find_many(vec![
			file_path::location_id::equals(location.id),
			file_path::parent_id::equals(Some(directory.id)),
		])
		.with(file_path::file::fetch())
		.exec()
		.await?
		.into_iter()
		.map(Into::into)
		.collect();

	for file_path in &mut file_paths {
		if let Some(file) = &mut file_path.file {
			let thumb_path = Path::new(&config.data_path)
				.join(THUMBNAIL_CACHE_DIR_NAME)
				.join(format!("{}", location.id))
				.join(file.cas_id.clone())
				.with_extension("webp");

			file.has_thumbnail = thumb_path.exists();
		}
	}

	Ok(DirectoryWithContents {
		directory: directory.into(),
		contents: file_paths,
	})
}

pub async fn open_album(ctx: &CoreContext, album_id: i32) -> Result<AlbumWithFiles, AlbumError> {
	let db = &ctx.database;

	let album: Album = db
		.album()
		.find_unique(album::id::equals(album_id))
		.exec()
		.await?
		.ok_or_else(|| AlbumError::AlbumNotFound(album_id))?
		.into();

	let files_in_album: Vec<FileInAlbum> = db
		.file_in_album()
		.find_many(vec![file_in_album::album_id::equals(album_id)])
		.exec()
		.await?
		.into_iter()
		.map(Into::into)
		.collect();

	Ok(AlbumWithFiles {
		album,
		files_in_album,
	})
}
