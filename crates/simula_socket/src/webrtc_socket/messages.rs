use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub(crate) type PeerId = Uuid;

/// Events go from signalling server to peer
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PeerEvent {
    NewPeer(PeerId),
    PeerLeft(PeerId),
    Signal { sender: PeerId, data: PeerSignal },
}

// TODO: move back into lib
/// Requests go from peer to signalling server
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PeerRequest {
    Uuid(PeerId),
    Signal { receiver: PeerId, data: PeerSignal },
    KeepAlive,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PeerSignal {
    IceCandidate(String),
    Offer(String),
    Answer(String),
}
