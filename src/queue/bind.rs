use amqpr_codec::{Frame, FrameHeader, FramePayload, AmqpString};
use amqpr_codec::method::MethodPayload;
use amqpr_codec::method::queue::{QueueClass, BindMethod};

use futures::sink::{Sink, Send};

use std::collections::HashMap;


pub type QueueBound<S> = Send<S>;


/// Bind a queue asynchronously.
/// That means we won't wait to receive `Declare-Ok` method after send `Declare` method.
pub fn bind_queue<S, E>(
    socket: S,
    channel_id: u16,
    option: BindQueueOption,
) -> QueueBound<S>
where
    S: Sink<SinkItem = Frame>
{
    let bind = BindMethod {
        reserved1: 0,
        queue: option.queue,
        exchange: option.exchange,
        routing_key: option.routing_key,
        no_wait: true,
        arguments: HashMap::new(),
    };

    let frame = Frame {
        header: FrameHeader { channel: channel_id },
        payload: FramePayload::Method(MethodPayload::Queue(QueueClass::Bind(bind))),
    };

    socket.send(frame)
}



pub struct BindQueueOption {
    pub queue: AmqpString,
    pub exchange: AmqpString,
    pub routing_key: AmqpString,
    pub is_no_wait: bool,
}
