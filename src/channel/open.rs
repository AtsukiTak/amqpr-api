use amqpr_codec::{Frame, FrameHeader, FramePayload};
use amqpr_codec::method::MethodPayload;
use amqpr_codec::method::channel::{ChannelClass, OpenMethod};

use futures::{Future, Stream, Sink, Poll, Async};

use std::borrow::Borrow;

use common::{send_and_receive, SendAndReceive};
use errors::*;


pub fn open_channel<In, Out, E>(income: In, outcome: Out, channel_id: u16) -> ChannelOpen<In, Out>
where
    In: Stream<Error = E>,
    Out: Sink<SinkItem = Frame, SinkError = E>,
    E: From<Error>,
    In::Item: Borrow<Frame>,
{
    let open = OpenMethod { reserved1: "".into() };
    let frame = Frame {
        header: FrameHeader { channel: channel_id },
        payload: FramePayload::Method(MethodPayload::Channel(ChannelClass::Open(open))),
    };

    let find_open_ok: fn(&Frame) -> bool = |frame| {
        frame
            .method()
            .and_then(|c| c.channel())
            .and_then(|m| m.open_ok())
            .is_some()
    };

    ChannelOpen { process: send_and_receive(frame, income, outcome, find_open_ok) }
}



pub struct ChannelOpen<In, Out>
where
    Out: Sink,
{
    process: SendAndReceive<In, Out, fn(&Frame) -> bool>,
}


impl<In, Out, E> Future for ChannelOpen<In, Out>
where
    In: Stream<Error = E>,
    Out: Sink<SinkError = E>,
    E: From<Error>,
    In::Item: Borrow<Frame>,
{
    type Item = (In, Out);
    type Error = E;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let (_open_ok, income, outcome) = try_ready!(self.process);
        Ok(Async::Ready((income, outcome)))
    }
}
