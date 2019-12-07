#![allow(dead_code)]

use std::io;

use tokio::net::{self, ToSocketAddrs};
use tokio::runtime;

pub mod server;
pub mod session;

pub fn run<A>(bind: A) -> io::Result<()>
where
    A: net::ToSocketAddrs,
{
    unimplemented!()
}
