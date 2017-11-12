use amqpr_codec::{Frame, FrameHeader, FramePayload};
use amqpr_codec::method::MethodPayload;
use amqpr_codec::method::exchange::{ExchangeClass, DeclareMethod};

use futures::{Future, Stream, Sink, Poll, Async};
use futures::sink::Send;

use std::collections::HashMap;
use std::borrow::Borrow;

use common::{send_and_receive, SendAndReceive};
use errors::*;


/// Declare an exchange with option.
/// If "is_no_wait" flag of option is active, you MUST set "income" argument.
/// If "is_no_wait" flag of option is inactibe, you don't need to set "income" argument, it will be
/// ignored.
///
/// # Panic
/// If "income" field is `None` when "is_no_wait" flag is active.
pub fn declare_exchange<In, Out, E>(
    income: Option<In>,
    outcome: Out,
    channel_id: u16,
    option: DeclareExchangeOption,
) -> ExchangeDeclared<In, Out>
where
    In: Stream<Error = E>,
    Out: Sink<SinkItem = Frame, SinkError = E>,
    E: From<Error>,
    In::Item: Borrow<Frame>,
{
    let no_wait = option.is_no_wait;

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

    if no_wait {
        ExchangeDeclared::NoWait(outcome.send(frame))
    } else {
        let find_declare_ok: fn(&Frame) -> bool = |frame| {
            frame
                .method()
                .and_then(|m| m.exchange())
                .and_then(|c| c.declare_ok())
                .is_some()
        };

        ExchangeDeclared::Wait(send_and_receive(frame, income.unwrap(), outcome, find_declare_ok))
    }
}



pub enum ExchangeDeclared<In, Out>
where
    Out: Sink,
{
    NoWait(Send<Out>),
    Wait(SendAndReceive<In, Out, fn(&Frame) -> bool>),
}


impl<In, Out, E> Future for ExchangeDeclared<In, Out>
where
    In: Stream<Error = E>,
    Out: Sink<SinkItem = Frame, SinkError = E>,
    E: From<Error>,
    In::Item: Borrow<Frame>,
{
    type Item = (Option<In>, Out);
    type Error = E;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        use self::ExchangeDeclared::*;
        match self {
            &mut NoWait(ref mut sending) => {
                let outcome = try_ready!(sending);
                Ok(Async::Ready((None, outcome)))
            }
            &mut Wait(ref mut process) => {
                let (_dec_ok, income, outcome) = try_ready!(process);
                Ok(Async::Ready((Some(income), outcome)))
            }
        }
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
