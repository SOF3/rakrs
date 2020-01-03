#![allow(dead_code)]

use std::io;

use tokio::net;

pub mod server;
pub mod session;

pub fn run<A>(_bind: A) -> io::Result<()>
where
    A: net::ToSocketAddrs,
{
    unimplemented!()
}
