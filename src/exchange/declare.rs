use amqpr_codec::{AmqpString, Frame, FrameHeader, FramePayload};
use amqpr_codec::method::MethodPayload;
use amqpr_codec::method::exchange::{DeclareMethod, ExchangeClass};

use futures::sink::{Send, Sink};

use std::collections::HashMap;

pub type ExchangeDeclared<S> = Send<S>;

/// Declare exchange asynchronously.
/// That means we won't wait to receive `Declare-Ok` method after send `Declare` method.
pub fn declare_exchange<S>(
    channel_id: u16,
    socket: S,
    option: DeclareExchangeOption,
) -> ExchangeDeclared<S>
where
    S: Sink<SinkItem = Frame>,
{
    let declare = DeclareMethod {
        reserved1: 0,
        exchange: option.name,
        typ: name_of_type(option.typ),
        passive: option.is_passive,
        durable: option.is_durable,
        auto_delete: option.is_auto_delete,
        internal: option.is_internal,
        no_wait: false,
        arguments: HashMap::new(),
    };

    let frame = Frame {
        header: FrameHeader {
            channel: channel_id,
        },
        payload: FramePayload::Method(MethodPayload::Exchange(ExchangeClass::Declare(declare))),
    };

    socket.send(frame)
}

#[derive(Debug, Clone)]
pub struct DeclareExchangeOption {
    pub name: AmqpString,
    pub typ: ExchangeType,
    pub is_passive: bool,
    pub is_durable: bool,
    pub is_auto_delete: bool,
    pub is_internal: bool,
}

#[derive(Debug, Clone)]
pub enum ExchangeType {
    Direct,
    Fanout,
    Topic,
    Headers,
}

fn name_of_type(typ: ExchangeType) -> AmqpString {
    match typ {
        ExchangeType::Direct => "direct".into(),
        ExchangeType::Fanout => "fanout".into(),
        ExchangeType::Topic => "topic".into(),
        ExchangeType::Headers => "headers".into(),
    }
}
