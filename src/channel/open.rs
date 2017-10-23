use amqpr_codec::{Frame, FrameHeader, FramePayload};
use amqpr_codec::method::MethodPayload;
use amqpr_codec::method::channel::{ChannelClass, OpenMethod};

use futures::{Future, Poll, Async};

use AmqpSocket;
use common::{send_and_receive, SendAndReceive};
use errors::*;


pub fn open_channel(socket: AmqpSocket, channel_id: u16) -> ChannelOpen {
    let open = OpenMethod { reserved1: "".into() };
    let frame = Frame {
        header: FrameHeader { channel: channel_id },
        payload: FramePayload::Method(MethodPayload::Channel(ChannelClass::Open(open))),
    };
    ChannelOpen { process: send_and_receive(socket, frame) }
}



pub struct ChannelOpen {
    process: SendAndReceive,
}


impl Future for ChannelOpen {
    type Item = AmqpSocket;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let (frame, socket) = try_ready!(self.process);
        frame.method()
             .and_then(|m| m.channel())
             .and_then(|c| c.open_ok())
             .ok_or(Error::from(ErrorKind::UnexpectedFrame))
             .map(move |_| Async::Ready(socket))
    }
}
