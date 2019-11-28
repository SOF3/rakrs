use std::future::Future;
use std::io;
use std::net::SocketAddr;

use rakrs_io::CanIo;
use rakrs_protocol::{offline, online};
use tokio::net::{self, UdpSocket};

pub async fn run_server<A, FPollR, FCkR, FOnR, FOffR>(
    bind: A,
    poll_send: impl Fn() -> FPollR,
    query_online: impl Fn(&'_ SocketAddr) -> FCkR,
    push_online: impl Fn(SocketAddr, online::OnlinePacket) -> FOnR,
    push_offline: impl Fn(SocketAddr, offline::OfflinePacket) -> FOffR,
) -> io::Result<()>
where
    A: net::ToSocketAddrs,
    FPollR: Future<Output = Option<(SocketAddr, Vec<u8>)>>, // TODO optimize the return type to reduce allocations
    FCkR: Future<Output = bool>,
    FOnR: Future<Output = ()>,
    FOffR: Future<Output = ()>,
{
    let mut socket = UdpSocket::bind(&bind).await?;

    loop {
        while let Some((addr, buf)) = poll_send().await {
            match socket.send_to(&buf[..], &addr).await {
                Ok(size) => {
                    if size != buf.len() {
                        log::warn!(
                            "Failed to write {} bytes to {}: only wrote {} bytes",
                            buf.len(),
                            &addr,
                            size
                        );
                    }
                }
                Err(err) => {
                    log::error!("Failed to write {} bytes to {}: {}", buf.len(), &addr, err);
                }
            }
        }

        let mut buf = [0; 65536];
        let (size, remote) = match socket.recv_from(&mut buf).await {
            Ok(pair) => pair,
            Err(err) => {
                log::error!("Error reading socket: {}", err);
                continue;
            }
        };
        let data = &buf[..size];

        let is_online = query_online(&remote).await;
        if is_online {
            match online::OnlinePacket::read(io::Cursor::new(data)) {
                Ok(Some(packet)) => push_online(remote, packet).await,
                Ok(None) => {
                    log::warn!("Received offline packet from connected session {}", &remote);
                }
                Err(err) => {
                    log::error!("Error parsing online packet from {}: {}", &remote, err);
                }
            }
        } else {
            match offline::OfflinePacket::read(io::Cursor::new(data)) {
                Ok(packet) => push_offline(remote, packet).await,
                Err(err) => {
                    log::error!("Error parsing offline packet from {}: {}", &remote, err);
                }
            }
        }
    }
}
