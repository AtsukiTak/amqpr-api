use amqpr_codec::{Frame, FrameHeader, FramePayload};
use amqpr_codec::content_body::ContentBodyPayload;
use amqpr_codec::content_header::ContentHeaderPayload;
use amqpr_codec::method::MethodPayload;
use amqpr_codec::method::basic::{BasicClass, PublishMethod};

use bytes::Bytes;

use futures::{Future, Sink, Poll, Async};
use futures::sink::Send;

use std::io::Error as IoError;

use AmqpSocket;
use common::Should;


pub fn publish<E>(
    socket: AmqpSocket,
    channel_id: u16,
    bytes: Bytes,
    option: PublishOption,
) -> Published<E>
where
    E: From<IoError>,
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
        state: SendingContentState::SendingPublishMethod(socket.send(frame)),
        bytes: Should::new(bytes),
        channel_id: channel_id,
        phantom: ::std::marker::PhantomData,
    }
}



pub struct PublishOption {
    pub exchange: String,
    pub routing_key: String,
    pub is_mandatory: bool,
    pub is_immediate: bool,
}



// Published struct {{{
pub struct Published<E>
where
    E: From<IoError>,
{
    state: SendingContentState,
    bytes: Should<Bytes>,
    channel_id: u16,
    phantom: ::std::marker::PhantomData<E>,
}

pub enum SendingContentState {
    SendingPublishMethod(Send<AmqpSocket>),
    SendingContentHeader(Send<AmqpSocket>),
    SendingContentBody(Send<AmqpSocket>),
}


impl<E> Future for Published<E>
where
    E: From<IoError>,
{
    type Item = AmqpSocket;
    type Error = E;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {

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
                debug!("Sending content header : {:?}", frame);
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
                debug!("Sending content body : {:?}", frame);
                SendingContentBody(socket.send(frame))
            }

            &mut SendingContentBody(ref mut sending) => {
                return Ok(Async::Ready(try_ready!(sending)));
            }
        };

        self.poll()
    }
}
// }}}
