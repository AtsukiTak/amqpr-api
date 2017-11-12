use amqpr_codec::{Frame, FrameHeader, FramePayload};
use amqpr_codec::method::MethodPayload;
use amqpr_codec::method::basic::{BasicClass, ConsumeMethod};

use futures::{Future, Stream, Sink, Poll, Async};

use std::collections::HashMap;
use std::borrow::Borrow;

use common::{send_and_receive, SendAndReceive};
use errors::*;


pub fn start_consume<In, Out, E>(
    income: In,
    outcome: Out,
    channel_id: u16,
    option: StartConsumeOption,
) -> ConsumeStarted<In, Out>
where
    In: Stream<Error = E>,
    In::Item: Borrow<Frame>,
    Out: Sink<SinkItem = Frame, SinkError = E>,
    E: From<Error>,
{
    let consume = ConsumeMethod {
        reserved1: 0,
        queue: option.queue,
        consumer_tag: option.consumer_tag,
        no_local: option.is_no_local,
        no_ack: option.is_no_ack,
        exclusive: option.is_exclusive,
        no_wait: option.is_no_wait,
        arguments: HashMap::new(),
    };

    let frame = Frame {
        header: FrameHeader { channel: channel_id },
        payload: FramePayload::Method(MethodPayload::Basic(BasicClass::Consume(consume))),
    };

    let find_consume_ok: fn(&Frame) -> bool = |frame| {
        frame
            .method()
            .and_then(|c| c.basic())
            .and_then(|m| m.consume_ok())
            .is_some()
    };

    let process = send_and_receive(frame, income, outcome, find_consume_ok);

    ConsumeStarted { process: process }
}


#[derive(Debug, Clone)]
pub struct StartConsumeOption {
    pub queue: String,
    pub consumer_tag: String,
    pub is_no_local: bool,
    pub is_no_ack: bool,
    pub is_exclusive: bool,
    pub is_no_wait: bool,
}



pub struct ConsumeStarted<In, Out>
where
    Out: Sink,
{
    process: SendAndReceive<In, Out, fn(&Frame) -> bool>,
}


impl<In, Out, E> Future for ConsumeStarted<In, Out>
where
    In: Stream<Error = E>,
    In::Item: Borrow<Frame>,
    Out: Sink<SinkItem = Frame, SinkError = E>,
    E: From<Error>,
{
    type Item = (In, Out);
    type Error = E;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let (_consume_ok, income, outcome) = try_ready!(self.process);
        Ok(Async::Ready((income, outcome)))
    }
}
