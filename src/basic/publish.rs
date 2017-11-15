use amqpr_codec::{Frame, FrameHeader, FramePayload};
use amqpr_codec::content_body::ContentBodyPayload;
use amqpr_codec::content_header::ContentHeaderPayload;
use amqpr_codec::method::MethodPayload;
use amqpr_codec::method::basic::{BasicClass, PublishMethod};

use bytes::Bytes;

use futures::{Future, Sink, Poll, Async};
use futures::sink::Send;

use common::Should;


pub fn publish<S>(sink: S, channel_id: u16, bytes: Bytes, option: PublishOption) -> Published<S>
where
    S: Sink<SinkItem = Frame>,
{
    let declare = PublishMethod {
        reserved1: 0,
        exchange: option.exchange,
        routing_key: option.routing_key,
        mandatory: option.is_mandatory,
        immediate: option.is_immediate,
    };

    let frame = Frame {
        header: FrameHeader { channel: channel_id },
        payload: FramePayload::Method(MethodPayload::Basic(BasicClass::Publish(declare))),
    };

    debug!("Sending publish method : {:?}", frame);
    Published {
        state: SendingContentState::SendingPublishMethod(sink.send(frame)),
        bytes: Should::new(bytes),
        channel_id: channel_id,
    }
}



#[derive(Clone, Debug)]
pub struct PublishOption {
    pub exchange: String,
    pub routing_key: String,
    pub is_mandatory: bool,
    pub is_immediate: bool,
}



// Published struct {{{
pub struct Published<S>
where
    S: Sink<SinkItem = Frame>,
{
    state: SendingContentState<S>,
    bytes: Should<Bytes>,
    channel_id: u16,
}

pub enum SendingContentState<S>
where
    S: Sink<SinkItem = Frame>,
{
    SendingPublishMethod(Send<S>),
    SendingContentHeader(Send<S>),
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
            &mut SendingPublishMethod(ref mut sending) => {
                let socket = try_ready!(sending);
                let frame = Frame {
                    header: FrameHeader { channel: self.channel_id },
                    payload: FramePayload::ContentHeader(ContentHeaderPayload {
                        class_id: 60,
                        body_size: self.bytes.as_ref().len() as u64,
                        property_flags: 1,
                    }),
                };
                debug!("Sent publish method");
                SendingContentHeader(socket.send(frame))
            }

            &mut SendingContentHeader(ref mut sending) => {
                let socket = try_ready!(sending);
                let frame = {
                    let payload = ContentBodyPayload { bytes: self.bytes.take() };
                    Frame {
                        header: FrameHeader { channel: self.channel_id },
                        payload: FramePayload::ContentBody(payload),
                    }
                };
                debug!("Sent content header");
                SendingContentBody(socket.send(frame))
            }

            &mut SendingContentBody(ref mut sending) => {
                let sink = try_ready!(sending);
                debug!("Sent content body");
                return Ok(Async::Ready(sink));
            }
        };

        self.poll()
    }
}
// }}}
