use amqpr_codec::{Frame, FrameHeader, FramePayload, AmqpString};
use amqpr_codec::content_body::ContentBodyPayload;
use amqpr_codec::content_header::{ContentHeaderPayload, Properties};
use amqpr_codec::method::MethodPayload;
use amqpr_codec::method::basic::{BasicClass, PublishMethod};

use bytes::Bytes;

use futures::{Future, Sink, Poll, Async};
use futures::sink::Send;

use common::Should;


/// Publish an item to AMQP server.
/// If you want to publish a lot number of items, please consider to use `publish_sink` function.
/// Returned item is `Future` which will be completed when finish to send.
pub fn publish<S>(sink: S, channel_id: u16, item: PublishItem) -> Published<S>
where
    S: Sink<SinkItem = Frame>,
{
    let (meta, header, body) = (item.meta, item.header, item.body);

    let declare = PublishMethod {
        reserved1: 0,
        exchange: meta.exchange,
        routing_key: meta.routing_key,
        mandatory: meta.is_mandatory,
        immediate: meta.is_immediate,
    };

    let frame = Frame {
        header: FrameHeader { channel: channel_id },
        payload: FramePayload::Method(MethodPayload::Basic(BasicClass::Publish(declare))),
    };

    debug!("Sending publish method : {:?}", frame);

    Published {
        state: SendingContentState::SendingPublishMethod(
            sink.send(frame),
            Should::new(header),
            Should::new(body),
        ),
        channel_id: channel_id,
    }
}



/// A meta option of `Publish` message on AMQP.
#[derive(Clone, Debug)]
pub struct PublishOption {
    pub exchange: AmqpString,
    pub routing_key: AmqpString,
    pub is_mandatory: bool,
    pub is_immediate: bool,
}


#[derive(Clone, Debug)]
pub struct PublishItem {
    pub meta: PublishOption,
    pub header: Properties,
    pub body: Bytes,
}



// Published struct {{{
pub struct Published<S>
where
    S: Sink<SinkItem = Frame>,
{
    state: SendingContentState<S>,
    channel_id: u16,
}

pub enum SendingContentState<S>
where
    S: Sink<SinkItem = Frame>,
{
    SendingPublishMethod(Send<S>, Should<Properties>, Should<Bytes>),
    SendingContentHeader(Send<S>, Should<Bytes>),
    SendingContentBody(Send<S>),
}


impl<S> Future for Published<S>
where
    S: Sink<SinkItem = Frame>,
{
    type Item = S;
    type Error = S::SinkError;

    fn poll(&mut self) -> Poll<S, S::SinkError> {

        use self::SendingContentState::*;
        self.state = match &mut self.state {
            &mut SendingPublishMethod(ref mut sending, ref mut properties, ref mut bytes) => {
                let socket = try_ready!(sending.poll());
                let header = ContentHeaderPayload {
                    class_id: 60,
                    body_size: bytes.as_ref().len() as u64,
                    properties: properties.take(),
                };
                let frame = Frame {
                    header: FrameHeader { channel: self.channel_id },
                    payload: FramePayload::ContentHeader(header),
                };
                debug!("Sent publish method");
                SendingContentHeader(socket.send(frame), bytes.clone())
            }

            &mut SendingContentHeader(ref mut sending, ref mut bytes) => {
                let socket = try_ready!(sending.poll());
                let frame = {
                    let payload = ContentBodyPayload { bytes: bytes.take() };
                    Frame {
                        header: FrameHeader { channel: self.channel_id },
                        payload: FramePayload::ContentBody(payload),
                    }
                };
                debug!("Sent content header");
                SendingContentBody(socket.send(frame))
            }

            &mut SendingContentBody(ref mut sending) => {
                let sink = try_ready!(sending.poll());
                debug!("Sent content body");
                return Ok(Async::Ready(sink));
            }
        };

        self.poll()
    }
}
// }}}
