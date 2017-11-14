use amqpr_codec::{Frame, FrameHeader, FramePayload};
use amqpr_codec::method::MethodPayload;
use amqpr_codec::method::queue::{QueueClass, BindMethod};

use futures::{Future, Stream, Sink, Poll, Async};

use std::collections::HashMap;
use std::borrow::Borrow;

use common::{send_and_receive, SendAndReceive};
use errors::*;


/// Bind a queue synchronously.
/// That means we will wait to receive `Declare-Ok` method after send `Declare` method.
/// If you want not to wait receiving, you should use `declare_queue` instead.
/// This function ignores `is_not_wait` flag of option.
pub fn bind_queue_wait<In, Out, E>(
    income: In,
    outcome: Out,
    channel_id: u16,
    option: BindQueueOption,
) -> QueueBound<In, Out>
where
    In: Stream<Error = E>,
    In::Item: Borrow<Frame>,
    Out: Sink<SinkItem = Frame, SinkError = E>,
    E: From<Error>,
{
    let bind = BindMethod {
        reserved1: 0,
        queue: option.queue,
        exchange: option.exchange,
        routing_key: option.routing_key,
        no_wait: false,
        arguments: HashMap::new(),
    };

    let frame = Frame {
        header: FrameHeader { channel: channel_id },
        payload: FramePayload::Method(MethodPayload::Queue(QueueClass::Bind(bind))),
    };

    let find_dec_ok: fn(&Frame) -> bool = |frame| {
        frame
            .method()
            .and_then(|m| m.queue())
            .and_then(|c| c.bind_ok())
            .is_some()
    };

    QueueBound { process: send_and_receive(frame, income, outcome, find_dec_ok) }
}



pub struct BindQueueOption {
    pub queue: String,
    pub exchange: String,
    pub routing_key: String,
    pub is_no_wait: bool,
}




pub struct QueueBound<In, Out>
where
    Out: Sink,
{
    process: SendAndReceive<In, Out, fn(&Frame) -> bool>,
}


impl<In, Out, E> Future for QueueBound<In, Out>
where
    In: Stream<Error = E>,
    In::Item: Borrow<Frame>,
    Out: Sink<SinkItem = Frame, SinkError = E>,
    E: From<Error>,
{
    type Item = (In, Out);
    type Error = E;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let (_frame, income, outcome) = try_ready!(self.process);
        Ok(Async::Ready((income, outcome)))
    }
}


#[derive(Debug, Clone)]
pub struct DeclareQueueOption {
    pub name: String,
    pub is_passive: bool,
    pub is_durable: bool,
    pub is_exclusive: bool,
    pub is_auto_delete: bool,
    pub is_no_wait: bool,
}
