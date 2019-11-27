use std::net::SocketAddr;

use send_queue::SendQueue;

mod send_queue;

pub struct Session {
    address: SocketAddr,
    send_queue: SendQueue,
    state: SessionState,
}

pub enum SessionState {}
