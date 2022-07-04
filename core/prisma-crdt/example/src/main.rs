#![allow(unused)]

mod prisma;
mod prisma_crdt;

use ::prisma_crdt::CRDTOperation;
use prisma::PrismaClient;
// use serde_json::json;

use crate::prisma_crdt::new_client;
use serde_json::json;

#[tokio::main]
async fn main() {
	let client = prisma::new_client().await.unwrap();

	let node_0 = client
		.node()
		.upsert(
			prisma_crdt::node::id::equals(vec![0]),
			(vec![0], "Node 0".to_string(), vec![]),
			vec![],
		)
		.exec()
		.await
		.unwrap();

	let node_1 = client
		.node()
		.upsert(
			prisma_crdt::node::id::equals(vec![1]),
			(vec![1], "Node 1".to_string(), vec![]),
			vec![],
		)
		.exec()
		.await
		.unwrap();

	producer_example(client, node_0).await;
	// consumer_example(client, node_1).await;
}

async fn producer_example(client: PrismaClient, node: prisma::node::Data) {
	let (client, mut op_receiver) = new_client(client, node.id.clone(), node.local_id).await;

	let task = tokio::spawn(async move {
		while let Some(op) = op_receiver.recv().await {
			println!("{}", serde_json::to_string_pretty(&op).unwrap());
		}
	});

	let location = client
		.location()
		.create(vec![0], "Location 0".to_string(), vec![])
		.exec()
		.await
		.unwrap();

	let data = client
		.file_path()
		.create(0, location.local_id, "File 0".to_string(), vec![])
		.exec()
		.await
		.unwrap();

	let file = client
		.file()
		.create(vec![0], vec![prisma_crdt::file::size_in_bytes::set(100)])
		.exec()
		.await
		.unwrap();

	client
		.file_path()
		.update(
			prisma_crdt::file_path::location_id_id(data.location_id, data.id),
			vec![prisma_crdt::file_path::file_id::set(Some(file.local_id))],
		)
		.exec()
		.await
		.unwrap();

	let tag = client
		.tag()
		.create(vec![0], "Tag 0".to_string(), vec![])
		.exec()
		.await
		.unwrap();

	client
		.tag_on_file()
		.create(tag.local_id, file.local_id, vec![])
		.exec()
		.await
		.unwrap();

	dbg!(
		client
			.file()
			.find_many(vec![prisma::file::tag_on_file::some(vec![
				prisma::tag_on_file::tag_id::equals(tag.local_id)
			])])
			.exec()
			.await
	);
}

async fn consumer_example(client: PrismaClient, node: prisma::node::Data) {
	let (client, mut op_receiver) = new_client(client, node.id.clone(), node.local_id).await;

	client
		._execute_operation(
			serde_json::from_value(json!({
				"n": [0],
				"t": 0,
				"m": "Location",
				"d": [{
					"c": {
						"id": [0],
						"name": "Location 0"
					}
				}]
			}))
			.unwrap(),
		)
		.await;

	client
		._execute_operation(
			serde_json::from_value(json!({
				"n": [0],
				"t": 0,
				"m": "FilePath",
				"d": [{
					"c": {
						"id": 0,
						"location_id": [0],
						"name": "File 0"
					}
				}]
			}))
			.unwrap(),
		)
		.await;

	client
		._execute_operation(
			serde_json::from_value(json!({
			  "n": [0],
			  "t": 0,
			  "r": [0],
			  "m": "File",
			  "c": "a"
			}))
			.unwrap(),
		)
		.await;

	client
		._execute_operation(
			serde_json::from_value(json!({
			  "n": [0],
			  "t": 0,
			  "r": [0],
			  "m": "File",
			  "u": {
				"f": "size_in_bytes",
				"v": 100
			  }
			}))
			.unwrap(),
		)
		.await;

	client
		._execute_operation(
			serde_json::from_value(json!({
				"n": [0],
				"t": 0,
				"m": "FilePath",
				"d": [{
					"u": {
						"id": 0,
						"location_id": [0],
						"_": [{
							"file_id": 1
						}]
					}
				}]
			}))
			.unwrap(),
		)
		.await;

	client
		._execute_operation(
			serde_json::from_value(json!({
			  "n": [0],
			  "t": 0,
			  "r": [0],
			  "m": "Tag",
			  "c": {
				"u": {
				  "id": [0],
				  "name": "Tag 0"
				}
			  }
			}))
			.unwrap(),
		)
		.await;

	client
		._execute_operation(
			serde_json::from_value(json!({
			  "n": [0],
			  "t": 0,
			  "relation": "TagOnFile",
			  "relation_item": [0],
			  "relation_group": [0],
			  "type": "c"
			}))
			.unwrap(),
		)
		.await;

	dbg!(client.location().find_many(vec![]).exec().await);
	dbg!(client.file_path().find_many(vec![]).exec().await);
	dbg!(client.file().find_many(vec![]).exec().await);
	dbg!(client.tag().find_many(vec![]).exec().await);
}
