mod ast;
mod attribute_parser;
mod client;

use prisma_client_rust_sdk::PrismaGenerator;
use quote::quote;

pub const INTERNAL_MODELS: &'static [&'static str] =
	&["OwnedOperation", "SharedOperation", "RelationOperation"];

pub struct PrismaCRDTGenerator;

impl PrismaGenerator for PrismaCRDTGenerator {
	const NAME: &'static str = "Prisma CRDT Generator";
	const DEFAULT_OUTPUT: &'static str = "./prisma-crdt.rs";

	fn generate(args: prisma_client_rust_sdk::GenerateArgs) -> String {
		let mut out = String::new();

		let header = quote! {
			pub async fn new_client(
				prisma_client: crate::prisma::PrismaClient,
				node_id: Vec<u8>,
				node_local_id: i32
			) -> (
				_prisma::prismaCRDTClient,
				::tokio::sync::mpsc::Receiver<::prisma_crdt::CRDTOperation>,
			) {
				let (tx, rx) = ::tokio::sync::mpsc::channel(64);

				let crdt_client = _prisma::PrismaCRDTClient::_new(prisma_client, (node_id, node_local_id), tx);
				(crdt_client, rx)
			}
			pub use _prisma::*;
		};

		let client = client::generate(&args);

		let output = quote! {
			#header
			#client
		};

		output.to_string()
	}
}
