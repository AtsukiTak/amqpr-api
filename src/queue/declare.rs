use amqpr_codec::{Frame, FrameHeader, FramePayload};
use amqpr_codec::method::MethodPayload;
use amqpr_codec::method::queue::{QueueClass, DeclareMethod};
pub use amqpr_codec::method::queue::DeclareOkMethod as DeclareResult;

use futures::{Future, Stream, Sink, Poll, Async};

use std::collections::HashMap;
use std::borrow::Borrow;

use common::{send_and_receive, SendAndReceive};
use errors::*;


/// Declare a queue synchronously.
/// That means we will wait to receive `Declare-Ok` method after send `Declare` method.
/// If you want not to wait receiving, you should use `declare_queue` instead.
/// This function ignores `is_not_wait` flag of option.
pub fn declare_queue_wait<In, Out, E>(
    income: In,
    outcome: Out,
    channel_id: u16,
    option: DeclareQueueOption,
) -> QueueDeclaredWait<In, Out>
where
    In: Stream<Error = E>,
    In::Item: Borrow<Frame>,
    Out: Sink<SinkItem = Frame, SinkError = E>,
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

    let find_dec_ok: fn(&Frame) -> bool = |frame| {
        frame
            .method()
            .and_then(|m| m.queue())
            .and_then(|c| c.declare_ok())
            .is_some()
    };
    QueueDeclaredWait { process: send_and_receive(frame, income, outcome, find_dec_ok) }
}



pub struct QueueDeclaredWait<In, Out>
where
    Out: Sink,
{
    process: SendAndReceive<In, Out, fn(&Frame) -> bool>,
}


impl<In, Out, E> Future for QueueDeclaredWait<In, Out>
where
    In: Stream<Error = E>,
    In::Item: Borrow<Frame>,
    Out: Sink<SinkItem = Frame, SinkError = E>,
    E: From<Error>,
{
    type Item = (DeclareResult, In, Out);
    type Error = E;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let (frame, income, outcome) = try_ready!(self.process);
        let dec_ok = frame
            .borrow()
            .method()
            .and_then(|m| m.queue())
            .and_then(|c| c.declare_ok())
            .unwrap()
            .clone();
        debug!("Receive declare-ok response");
        Ok(Async::Ready((dec_ok, income, outcome)))
    }
}


#[derive(Clone, Debug)]
pub struct DeclareQueueOption {
    pub name: String,
    pub is_passive: bool,
    pub is_durable: bool,
    pub is_exclusive: bool,
    pub is_auto_delete: bool,
    pub is_no_wait: bool,
}
