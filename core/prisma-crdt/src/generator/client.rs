use prisma_client_rust_sdk::GenerateArgs;
use proc_macro2::TokenStream;
use quote::quote;

use super::{attribute_parser::attribute, INTERNAL_MODELS};

fn create_operation_fn() -> TokenStream {
	quote! {
		pub async fn _create_operation(&self, typ: ::prisma_crdt::CRDTOperationType) {
			let timestamp = ::uhlc::NTP64(0); // TODO: actual timestamps

			let timestamp_bytes = vec![0];

			use prisma_crdt::*;

			match &typ {
				CRDTOperationType::Shared(SharedOperation {
					record_id,
					model,
					data,
				}) => {
					let (kind, data) = match data {
						SharedOperationData::Create(typ) => {
							("c".to_string(), serde_json::to_vec(typ).unwrap())
						}
						SharedOperationData::Update { field, value } => {
							("u".to_string() + field, serde_json::to_vec(value).unwrap())
						}
						SharedOperationData::Delete => ("d".to_string(), vec![]),
					};

					self.client
						.shared_operation()
						.create(
							timestamp_bytes,
							::serde_json::to_vec(&record_id).unwrap(),
							kind,
							model.to_string(),
							data,
							crate::prisma::node::local_id::equals(self.node_local_id),
							vec![],
						)
						.exec()
						.await;
				}
				CRDTOperationType::Owned(op) => {
					self.client
						.owned_operation()
						.create(
							timestamp_bytes,
							serde_json::to_vec(op).unwrap(),
							crate::prisma::node::local_id::equals(self.node_local_id),
							vec![],
						)
						.exec()
						.await;
				}
				CRDTOperationType::Relation(RelationOperation {
					relation,
					relation_item,
					relation_group,
					data,
				}) => {
					let (kind, data) = match data {
						RelationOperationData::Create => ("c".to_string(), vec![]),
						RelationOperationData::Update { field, value } => {
							("u".to_string() + field, serde_json::to_vec(value).unwrap())
						}
						RelationOperationData::Delete => ("d".to_string(), vec![]),
					};

					self.client
						.relation_operation()
						.create(
							timestamp_bytes,
							relation.to_string(),
							relation_item.clone(),
							relation_group.clone(),
							kind,
							data,
							crate::prisma::node::local_id::equals(self.node_local_id),
							vec![],
						)
						.exec()
						.await;
				}
			}

			let op = CRDTOperation::new(self.node_id.clone(), timestamp, typ);

			self.operation_sender.send(op).await;
		}
	}
}

fn actions_accessors(args: &GenerateArgs) -> TokenStream {
	let actions_accessors = args
		.dml
		.models()
		.filter(|m| !INTERNAL_MODELS.contains(&m.name.as_str()))
		.map(|m| quote!());

	quote! {
		#(#actions_accessors)*
	}
}

pub fn generate(args: &GenerateArgs) -> TokenStream {
	let create_operation_fn = create_operation_fn();

	let attrs = args
		.dml
		.models
		.iter()
		.map(|m| m.documentation.as_ref().map(|d| attribute(d).unwrap().1))
		.collect::<Vec<_>>();

	std::fs::write("./attrs.rs", format!("{attrs:#?}"));

	quote! {
		pub struct PrismaCRDTClient {
			pub(super) client: crate::prisma::PrismaClient,
			pub node_id: Vec<u8>,
			pub node_local_id: i32,
			operation_sender: ::tokio::sync::mpsc::Sender<::prisma_crdt::CRDTOperation>,
		}

		impl PrismaCRDTClient {
			pub(super) fn _new(
				client: crate::prisma::PrismaClient,
				(node_id, node_local_id): (Vec<u8>, i32),
				operation_sender: tokio::sync::mpsc::Sender<prisma_crdt::CRDTOperation>,
			) -> Self {
				Self {
					client,
					operation_sender,
					node_id,
					node_local_id,
				}
			}

			#create_operation_fn
		}

		pub async fn _create_operation(&self, typ: ::prisma_crdt::CRDTOperationType) {
			let timestamp = ::uhlc::NTP64(0); // TODO: actual timestamps

			let timestamp_bytes = vec![0];

			use prisma_crdt::*;

			match &typ {
				CRDTOperationType::Shared(SharedOperation {
					record_id,
					model,
					data,
				}) => {
					let (kind, data) = match data {
						SharedOperationData::Create(typ) => {
							("c".to_string(), serde_json::to_vec(typ).unwrap())
						}
						SharedOperationData::Update { field, value } => {
							("u".to_string() + field, serde_json::to_vec(value).unwrap())
						}
						SharedOperationData::Delete => ("d".to_string(), vec![]),
					};

					self.client
						.shared_operation()
						.create(
							timestamp_bytes,
							::serde_json::to_vec(&record_id).unwrap(),
							kind,
							model.to_string(),
							data,
							crate::prisma::node::local_id::equals(self.node_local_id),
							vec![],
						)
						.exec()
						.await;
				}
				CRDTOperationType::Owned(op) => {
					self.client
						.owned_operation()
						.create(
							timestamp_bytes,
							serde_json::to_vec(op).unwrap(),
							crate::prisma::node::local_id::equals(self.node_local_id),
							vec![],
						)
						.exec()
						.await;
				}
				CRDTOperationType::Relation(RelationOperation {
					relation,
					relation_item,
					relation_group,
					data,
				}) => {
					let (kind, data) = match data {
						RelationOperationData::Create => ("c".to_string(), vec![]),
						RelationOperationData::Update { field, value } => {
							("u".to_string() + field, serde_json::to_vec(value).unwrap())
						}
						RelationOperationData::Delete => ("d".to_string(), vec![]),
					};

					self.client
						.relation_operation()
						.create(
							timestamp_bytes,
							relation.to_string(),
							relation_item.clone(),
							relation_group.clone(),
							kind,
							data,
							crate::prisma::node::local_id::equals(self.node_local_id),
							vec![],
						)
						.exec()
						.await;
				}
			}

			let op = CRDTOperation::new(self.node_id.clone(), timestamp, typ);

			self.operation_sender.send(op).await;
		}
	}
}
