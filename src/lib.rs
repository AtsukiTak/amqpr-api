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
pub use basic::{publish, get_delivered, start_consume};
pub use subscribe_stream::subscribe_stream;
pub use publish_sink::publish_sink;

use futures::{Async, AsyncSink, Stream, Sink, Poll, StartSend};
use errors::Error;
use amqpr_codec::Frame;


type RawSocket = tokio_io::codec::Framed<tokio_core::net::TcpStream, amqpr_codec::Codec>;

pub struct AmqpSocket(RawSocket);

impl Stream for AmqpSocket {
    type Item = Frame;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        self.0.poll().map_err(|io_err| Error::from(io_err))
    }
}

impl Sink for AmqpSocket {
    type SinkItem = Frame;
    type SinkError = Error;

    fn start_send(&mut self, item: Frame) -> StartSend<Frame, Error> {
        self.0.start_send(item).map_err(|io_err| Error::from(io_err))
    }

    fn poll_complete(&mut self) -> Poll<(), Error> {
        self.0.poll_complete().map_err(|io_err| Error::from(io_err))
    }

    fn close(&mut self) -> Poll<(), Error> {
        self.0.close().map_err(|io_err| Error::from(io_err))
    }
}


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

    fn start_send(
        &mut self,
        item: Out::SinkItem,
    ) -> Result<AsyncSink<Out::SinkItem>, Out::SinkError> {
        self.1.start_send(item)
    }

    fn poll_complete(&mut self) -> Result<Async<()>, Out::SinkError> {
        self.1.poll_complete()
    }

    fn close(&mut self) -> Result<Async<()>, Out::SinkError> {
        self.1.close()
    }
}
