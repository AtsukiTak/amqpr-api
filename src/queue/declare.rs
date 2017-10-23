use amqpr_codec::{Frame, FrameHeader, FramePayload};
use amqpr_codec::method::MethodPayload;
use amqpr_codec::method::queue::{QueueClass, DeclareMethod};
pub use amqpr_codec::method::queue::DeclareOkMethod as DeclareResult;

use futures::{Future, Poll, Async};

use std::collections::HashMap;

use AmqpSocket;
use common::{send_and_receive, SendAndReceive};
use errors::*;


pub fn declare_queue(
    socket: AmqpSocket,
    channel_id: u16,
    option: DeclareQueueOption,
) -> QueueDeclared {
    let declare = DeclareMethod {
        reserved1: 0,
        queue: option.name,
        passive: option.is_passive,
        durable: option.is_durable,
        exclusive: option.is_exclusive,
        auto_delete: option.is_auto_delete,
        no_wait: option.is_no_wait,
        arguments: HashMap::new(),
    };

    let frame = Frame {
        header: FrameHeader { channel: channel_id },
        payload: FramePayload::Method(MethodPayload::Queue(QueueClass::Declare(declare))),
    };
    QueueDeclared { process: send_and_receive(socket, frame) }
}



pub struct QueueDeclared {
    process: SendAndReceive,
}


impl Future for QueueDeclared {
    type Item = (DeclareResult, AmqpSocket);
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let (frame, socket) = try_ready!(self.process);
        frame
            .method()
            .and_then(|m| m.queue())
            .and_then(|c| c.declare_ok())
            .ok_or(Error::from(ErrorKind::UnexpectedFrame))
            .map(move |r| {
                info!("Declare ok method is received : {:?}", r);
                Async::Ready((r.clone(), socket))
            })
    }
}


pub struct DeclareQueueOption {
    pub name: String,
    pub is_passive: bool,
    pub is_durable: bool,
    pub is_exclusive: bool,
    pub is_auto_delete: bool,
    pub is_no_wait: bool,
}
