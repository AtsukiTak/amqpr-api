use amqpr_codec::{Frame, FrameHeader, FramePayload, AmqpString};
use amqpr_codec::method::MethodPayload;
use amqpr_codec::method::queue::{QueueClass, DeclareMethod};
pub use amqpr_codec::method::queue::DeclareOkMethod as DeclareResult;

use futures::{Future, Stream, Sink, Poll, Async};
use futures::sink::Send;

use std::collections::HashMap;

use common::Should;
use errors::*;


/// Declare a queue synchronously.
/// That means we will wait to receive `Declare-Ok` method after send `Declare` method.
pub fn declare_queue<S, E>(
    channel_id: u16,
    socket: S,
    option: DeclareQueueOption,
) -> QueueDeclared<S, E>
where
    S: Stream<Item = Frame, Error = E> + Sink<SinkItem = Frame, SinkError = E>,
    E: From<Error>,
{
    let declare = DeclareMethod {
        reserved1: 0,
        queue: option.name,
        passive: option.is_passive,
        durable: option.is_durable,
        exclusive: option.is_exclusive,
        auto_delete: option.is_auto_delete,
        no_wait: false,
        arguments: HashMap::new(),
    };

    let frame = Frame {
        header: FrameHeader { channel: channel_id },
        payload: FramePayload::Method(MethodPayload::Queue(QueueClass::Declare(declare))),
    };

    QueueDeclared::Sending(socket.send(frame))
}



pub enum QueueDeclared<S, E>
where
    S: Stream<Item = Frame, Error = E> + Sink<SinkItem = Frame, SinkError = E>,
    E: From<Error>,
{
    Sending(Send<S>),
    Receiveing(Should<S>),
}


impl<S, E> Future for QueueDeclared<S, E>
where
    S: Stream<Item = Frame, Error = E>
        + Sink<SinkItem = Frame, SinkError = E>,
    E: From<Error>,
{
    type Item = (DeclareResult, S);
    type Error = E;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        use self::QueueDeclared::*;

        let state = match self {
            &mut Sending(ref mut sending) => {
                let socket = try_ready!(sending.poll());
                Receiveing(Should::new(socket))
            }
            &mut Receiveing(ref mut socket) => {
                let frame = try_stream_ready!(socket.as_mut().poll());
                let dec_ok = match frame.method().and_then(|m| m.queue()).and_then(
                    |c| c.declare_ok(),
                ) {
                    Some(dec_ok) => dec_ok.clone(),
                    None => {
                        return Err(E::from(Error::from(ErrorKind::UnexpectedFrame(
                            "DeclareOk".into(),
                            frame.clone(),
                        ))))
                    }
                };
                debug!("Receive declare-ok response");

                return Ok(Async::Ready((dec_ok, socket.take())));
            }
        };

        *self = state;

        self.poll()
    }
}


#[derive(Clone, Debug)]
pub struct DeclareQueueOption {
    pub name: AmqpString,
    pub is_passive: bool,
    pub is_durable: bool,
    pub is_exclusive: bool,
    pub is_auto_delete: bool,
}
