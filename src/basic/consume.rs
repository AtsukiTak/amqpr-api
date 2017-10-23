use amqpr_codec::{Frame, FrameHeader, FramePayload};
use amqpr_codec::method::MethodPayload;
use amqpr_codec::method::basic::{BasicClass, ConsumeMethod};

use futures::{Future, Poll, Async};

use std::collections::HashMap;

use AmqpSocket;
use common::{send_and_receive, SendAndReceive};
use errors::*;


pub fn start_consume(
    socket: AmqpSocket,
    channel_id: u16,
    option: StartConsumeOption,
) -> ConsumeStarted {
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
    ConsumeStarted { process: send_and_receive(socket, frame) }
}



pub struct ConsumeStarted {
    process: SendAndReceive,
}


impl Future for ConsumeStarted {
    type Item = AmqpSocket;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let (frame, socket) = try_ready!(self.process);
        frame
            .method()
            .and_then(|m| m.basic())
            .and_then(|c| c.consume_ok())
            .ok_or(Error::from(ErrorKind::UnexpectedFrame))
            .map(move |_| {
                info!("Consume is started");
                Async::Ready(socket)
            })
    }
}


pub struct StartConsumeOption {
    pub queue: String,
    pub consumer_tag: String,
    pub is_no_local: bool,
    pub is_no_ack: bool,
    pub is_exclusive: bool,
    pub is_no_wait: bool,
}
