use amqpr_codec::{Frame, FrameHeader, FramePayload};
use amqpr_codec::method::MethodPayload;
use amqpr_codec::method::exchange::{ExchangeClass, DeclareMethod};

use futures::{Future, Stream, Sink, Poll, Async};

use std::collections::HashMap;
use std::borrow::Borrow;

use common::{send_and_receive, SendAndReceive};
use errors::*;


/// Declare exchange synchronously.
/// That means we will wait to receive `Declare-Ok` method after send `Declare` method.
/// If you want not to wait receiving, you should use `declare_exchange` instead.
/// This function ignore "is_no_wait" flag of option.
pub fn declare_exchange_wait<In, Out, E>(
    income: In,
    outcome: Out,
    channel_id: u16,
    option: DeclareExchangeOption,
) -> ExchangeDeclaredWait<In, Out>
where
    In: Stream<Error = E>,
    Out: Sink<SinkItem = Frame, SinkError = E>,
    E: From<Error>,
    In::Item: Borrow<Frame>,
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
        header: FrameHeader { channel: channel_id },
        payload: FramePayload::Method(MethodPayload::Exchange(ExchangeClass::Declare(declare))),
    };

    let find_declare_ok: fn(&Frame) -> bool = |frame| {
        frame
            .method()
            .and_then(|m| m.exchange())
            .and_then(|c| c.declare_ok())
            .is_some()
    };

    ExchangeDeclaredWait { process: send_and_receive(frame, income, outcome, find_declare_ok) }
}



pub struct ExchangeDeclaredWait<In, Out>
where
    Out: Sink,
{
    process: SendAndReceive<In, Out, fn(&Frame) -> bool>,
}


impl<In, Out, E> Future for ExchangeDeclaredWait<In, Out>
where
    In: Stream<Error = E>,
    Out: Sink<SinkItem = Frame, SinkError = E>,
    E: From<Error>,
    In::Item: Borrow<Frame>,
{
    type Item = (In, Out);
    type Error = E;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let (_dec_ok, income, outcome) = try_ready!(self.process);
        Ok(Async::Ready((income, outcome)))
    }
}


#[derive(Debug, Clone)]
pub struct DeclareExchangeOption {
    pub name: String,
    pub typ: ExchangeType,
    pub is_passive: bool,
    pub is_durable: bool,
    pub is_auto_delete: bool,
    pub is_internal: bool,
    pub is_no_wait: bool,
}

#[derive(Debug, Clone)]
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
