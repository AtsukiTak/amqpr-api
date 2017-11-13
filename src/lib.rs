//! amqpr-api is AMQP client api library.
//! You can talk with AMQP server via channel controller provided by this crate.
//! There is two kind of channel controllers; GlobalChannelController and LocalChannelController.
//!

extern crate tokio_core;
extern crate tokio_io;
extern crate futures;
extern crate bytes;
#[macro_use]
extern crate log;
#[macro_use]
extern crate error_chain;

extern crate amqpr_codec;


macro_rules! poll_item {
    ($socket: expr) => {
        match $socket.poll() {
            Ok(::futures::Async::Ready(Some(frame))) => frame,
            Ok(::futures::Async::Ready(None)) =>
                return Err(::errors::Error::from(::errors::ErrorKind::UnexpectedConnectionClose).into()),
            Ok(::futures::Async::NotReady) => return Ok(::futures::Async::NotReady),
            Err(e) => return Err(e.into()),
        }
    }
}

macro_rules! try_ready {
    ($future: expr) => {
        match $future.poll() {
            Ok(::futures::Async::Ready(item)) => item,
            Ok(::futures::Async::NotReady) => return Ok(::futures::Async::NotReady),
            Err(e) => return Err(e.into()),
        }
    }
}


pub mod channel;
pub mod exchange;
pub mod queue;
pub mod basic;
pub mod heartbeat;

pub mod handshake;
pub mod errors;
pub(crate) mod common;


pub use handshake::start_handshake;
pub use channel::open_channel;
pub use exchange::declare_exchange_wait;
pub use queue::{declare_queue_wait, bind_queue_wait};
pub use basic::{publish, receive_delivered, start_consume};
pub use heartbeat::send_heartbeat;


pub type AmqpSocket = tokio_io::codec::Framed<tokio_core::net::TcpStream, amqpr_codec::Codec>;
