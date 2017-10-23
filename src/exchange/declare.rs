use amqpr_codec::{Frame, FrameHeader, FramePayload};
use amqpr_codec::method::MethodPayload;
use amqpr_codec::method::exchange::{ExchangeClass, DeclareMethod};

use futures::{Future, Poll, Async};

use std::collections::HashMap;

use AmqpSocket;
use common::{send_and_receive, SendAndReceive};
use errors::*;


pub fn declare_exchange(
    socket: AmqpSocket,
    channel_id: u16,
    option: DeclareExchangeOption,
) -> ExchangeDeclared {
    let declare = DeclareMethod {
        reserved1: 0,
        exchange: option.name,
        typ: name_of_type(option.typ),
        passive: option.is_passive,
        durable: option.is_durable,
        auto_delete: option.is_auto_delete,
        internal: option.is_internal,
        no_wait: option.is_no_wait,
        arguments: HashMap::new(),
    };

    let frame = Frame {
        header: FrameHeader { channel: channel_id },
        payload: FramePayload::Method(MethodPayload::Exchange(ExchangeClass::Declare(declare))),
    };
    ExchangeDeclared { process: send_and_receive(socket, frame) }
}



pub struct ExchangeDeclared {
    process: SendAndReceive,
}


impl Future for ExchangeDeclared {
    type Item = AmqpSocket;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let (frame, socket) = try_ready!(self.process);
        frame
            .method()
            .and_then(|m| m.exchange())
            .and_then(|c| c.declare_ok())
            .ok_or(Error::from(ErrorKind::UnexpectedFrame))
            .map(move |_| {
                info!("Declared an exchange");
                Async::Ready(socket)
            })
    }
}


pub struct DeclareExchangeOption {
    pub name: String,
    pub typ: ExchangeType,
    pub is_passive: bool,
    pub is_durable: bool,
    pub is_auto_delete: bool,
    pub is_internal: bool,
    pub is_no_wait: bool,
}

pub enum ExchangeType {
    Direct,
    Fanout,
    Topic,
    Headers,
}

fn name_of_type(typ: ExchangeType) -> String {
    match typ {
        ExchangeType::Direct => "direct".into(),
        ExchangeType::Fanout => "fanout".into(),
        ExchangeType::Topic => "topic".into(),
        ExchangeType::Headers => "headers".into(),
    }
}
