use amqpr_codec::{Frame, FrameHeader, FramePayload};
use amqpr_codec::args::AmqpString;
use amqpr_codec::method::MethodPayload;
use amqpr_codec::method::basic::{BasicClass, ConsumeMethod};

use futures::sink::{Send, Sink};

use std::collections::HashMap;

use errors::*;

pub type ConsumeStarted<S> = Send<S>;

/// Send `Consume` message to AMQP server.
///
/// # Notice
/// A message being sent by this function is `no-wait` mode.
pub fn start_consume<S>(channel_id: u16, socket: S, option: StartConsumeOption) -> ConsumeStarted<S>
where
    S: Sink<SinkItem = Frame>,
    S::SinkError: From<Error>,
{
    let consume = ConsumeMethod {
        reserved1: 0,
        queue: option.queue,
        consumer_tag: option.consumer_tag,
        no_local: option.is_no_local,
        no_ack: option.is_no_ack,
        exclusive: option.is_exclusive,
        no_wait: true,
        arguments: HashMap::new(),
    };

    let frame = Frame {
        header: FrameHeader {
            channel: channel_id,
        },
        payload: FramePayload::Method(MethodPayload::Basic(BasicClass::Consume(consume))),
    };

    socket.send(frame)
}

#[derive(Debug, Clone)]
pub struct StartConsumeOption {
    pub queue: AmqpString,
    pub consumer_tag: AmqpString,
    pub is_no_local: bool,
    pub is_no_ack: bool,
    pub is_exclusive: bool,
}
