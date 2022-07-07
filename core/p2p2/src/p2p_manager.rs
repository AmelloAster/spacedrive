use std::{collections::HashMap, future::Future, pin::Pin, process::Output};

use quinn::{RecvStream, SendStream};
use sd_tunnel_utils::PeerId;

use crate::{NetworkManager, Peer, PeerMetadata};

/// TODO
pub trait P2PManager: Clone + Send + Sync + Sized + 'static {
	const APPLICATION_NAME: &'static str;

	/// is called to get the metadata of the application. This metadata is sent as part of the discovery payload.
	fn get_metadata(&self) -> PeerMetadata;

	/// TODO
	fn peer_discovered(&self, nm: &NetworkManager<Self>, peer_id: &PeerId) {}

	/// TODO
	fn peer_expired(&self, nm: &NetworkManager<Self>, peer_id: PeerId) {}

	/// TODO
	fn peer_connected(&self, nm: &NetworkManager<Self>, peer_id: PeerId) {}

	/// TODO
	fn peer_disconnected(&self, nm: &NetworkManager<Self>, peer_id: PeerId) {}

	/// TODO
	/// TODO: Error type
	fn peer_paired<'a>(
		&'a self,
		nm: &'a NetworkManager<Self>,
		peer_id: &'a PeerId,
		extra_data: &'a HashMap<String, String>,
	) -> Pin<Box<dyn Future<Output = Result<(), ()>> + Send + 'a>>;

	/// TODO
	fn peer_paired_rollback<'a>(
		&'a self,
		nm: &'a NetworkManager<Self>,
		peer_id: &'a PeerId,
		extra_data: &'a HashMap<String, String>,
	) -> Pin<Box<dyn Future<Output = ()> + Send + Sync + 'a>>;

	/// TODO
	fn accept_stream(&self, peer: &Peer<Self>, stream: (SendStream, RecvStream)) {}
}
