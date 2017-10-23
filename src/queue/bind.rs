use amqpr_codec::{Frame, FrameHeader, FramePayload};
use amqpr_codec::method::MethodPayload;
use amqpr_codec::method::queue::{QueueClass, BindMethod};

use futures::{Future, Poll, Async};

use std::collections::HashMap;

use AmqpSocket;
use common::{send_and_receive, SendAndReceive};
use errors::*;


pub fn bind_queue(socket: AmqpSocket, channel_id: u16, option: BindQueueOption) -> QueueBound {
    let bind = BindMethod {
        reserved1: 0,
        queue: option.queue,
        exchange: option.exchange,
        routing_key: option.routing_key,
        no_wait: option.is_no_wait,
        arguments: HashMap::new(),
    };

    let frame = Frame {
        header: FrameHeader { channel: channel_id },
        payload: FramePayload::Method(MethodPayload::Queue(QueueClass::Bind(bind))),
    };
    QueueBound { process: send_and_receive(socket, frame) }
}



pub struct QueueBound {
    process: SendAndReceive,
}


impl Future for QueueBound {
    type Item = AmqpSocket;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let (frame, socket) = try_ready!(self.process);
        frame.method()
             .and_then(|m| m.queue())
             .and_then(|c| c.bind_ok())
             .ok_or(Error::from(ErrorKind::UnexpectedFrame))
             .map(move |_| Async::Ready(socket))
    }
}


pub struct BindQueueOption {
    pub queue: String,
    pub exchange: String,
    pub routing_key: String,
    pub is_no_wait: bool,
}
