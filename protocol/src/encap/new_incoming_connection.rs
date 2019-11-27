use std::io::{Cursor, Error, ErrorKind, Read, Result, Write};
use std::net::SocketAddr;

use rakrs_io::CanIo;

#[derive(Clone, Debug, PartialEq)]
pub struct NewIncomingConnection {
    pub address: SocketAddr,
    pub system_addresses: Vec<SocketAddr>,
    pub send_ping_time: u64,
    pub send_pong_time: u64,
}

impl CanIo for NewIncomingConnection {
    fn write<W: Write>(&self, mut w: W) -> Result<()> {
        self.address.write(&mut w)?;
        for addr in &self.system_addresses {
            addr.write(&mut w)?;
        }
        self.send_ping_time.write(&mut w)?;
        self.send_pong_time.write(&mut w)?;
        Ok(())
    }

    fn read<R: Read>(mut r: R) -> Result<Self> {
        let address = <SocketAddr as CanIo>::read(&mut r)?;

        let mut buf = vec![];
        r.read_to_end(&mut buf)?; // a bit hacky
        drop(r);

        if buf.len() < 16 {
            return Err(Error::new(ErrorKind::UnexpectedEof, "Expected send_ping_time and send_pong_time"));
        }

        let sa_len = buf.len() - 16;
        let mut cursor = Cursor::new(&buf[sa_len..buf.len()]);
        let send_ping_time: u64 = CanIo::read(&mut cursor)?;
        let send_pong_time: u64 = CanIo::read(&mut cursor)?;

        let mut cursor = Cursor::new(&buf[0..sa_len]);
        let mut system_addresses = vec![];
        while (cursor.position() as usize) < sa_len {
            // Triggers whenever there is at least one remaining byte
            // so that UnexpectedEof will be thrown
            let addr = <SocketAddr as CanIo>::read(&mut cursor)?;
            system_addresses.push(addr);
        }

        Ok(Self { address, system_addresses, send_ping_time, send_pong_time })
    }
}
