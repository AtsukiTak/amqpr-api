//! amqpr-api is AMQP client api library.
//! You can talk with AMQP server via channel controller provided by this crate.
//! There is two kind of channel controllers; GlobalChannelController and LocalChannelController.
//!

extern crate tokio_core;
extern crate tokio_io;
#[macro_use]
extern crate futures;
extern crate bytes;
#[macro_use]
extern crate log;
#[macro_use]
extern crate error_chain;

extern crate amqpr_codec;


macro_rules! try_stream_ready {
    ($polled: expr) => {
        match $polled {
            Ok(::futures::Async::Ready(Some(frame))) => frame,
            Ok(::futures::Async::Ready(None)) =>
                return Err(::errors::Error::from(::errors::ErrorKind::UnexpectedConnectionClose).into()),
            Ok(::futures::Async::NotReady) => return Ok(::futures::Async::NotReady),
            Err(e) => return Err(e.into()),
        }
    }
}


pub mod channel;
pub mod exchange;
pub mod queue;
pub mod basic;
pub mod subscribe_stream;
pub mod publish_sink;

pub mod handshake;
pub mod errors;
pub(crate) mod common;


pub use handshake::start_handshake;
pub use channel::open_channel;
pub use exchange::declare_exchange;
pub use queue::{declare_queue, bind_queue};
pub use subscribe_stream::subscribe_stream;
pub use publish_sink::publish_sink;

use futures::{Async, AsyncSink, Stream, Sink};


pub type AmqpSocket = tokio_io::codec::Framed<tokio_core::net::TcpStream, amqpr_codec::Codec>;


/// This struct is useful when the case such as some functions require `S: Stream + Sink` but your socket is
/// separeted into `Stream` and `Sink`.
pub struct InOut<In: Stream, Out: Sink>(pub In, pub Out);

impl<In: Stream, Out: Sink> Stream for InOut<In, Out> {
    type Item = In::Item;
    type Error = In::Error;

    fn poll(&mut self) -> Result<Async<Option<In::Item>>, In::Error> {
        self.0.poll()
    }
}


impl<In: Stream, Out: Sink> Sink for InOut<In, Out> {
    type SinkItem = Out::SinkItem;
    type SinkError = Out::SinkError;

    fn start_send(&mut self, item: Out::SinkItem) -> Result<AsyncSink<Out::SinkItem>, Out::SinkError> {
        self.1.start_send(item)
    }

    fn poll_complete(&mut self) -> Result<Async<()>, Out::SinkError> {
        self.1.poll_complete()
    }

    fn close(&mut self) -> Result<Async<()>, Out::SinkError> {
        self.1.close()
    }
}
