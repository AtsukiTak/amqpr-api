use amqpr_codec::{Frame, FrameHeader, FramePayload};
use amqpr_codec::method::MethodPayload;
use amqpr_codec::method::channel::{ChannelClass, OpenMethod};

use futures::{Async, Future, Poll, Sink, Stream};
use futures::sink::Send;

use common::Should;
use errors::*;

/// Open new channel with given channel id.
///
/// Returned future consists of `Frame` and `S` but for now, `Frame` is meanless
/// because it has nothing.
pub fn open_channel<S, E>(channel_id: u16, socket: S) -> ChannelOpened<S>
where
    S: Stream<Item = Frame, Error = E> + Sink<SinkItem = Frame, SinkError = E>,
    E: From<Error>,
{
    let open = OpenMethod {
        reserved1: "".into(),
    };
    let frame = Frame {
        header: FrameHeader {
            channel: channel_id,
        },
        payload: FramePayload::Method(MethodPayload::Channel(ChannelClass::Open(open))),
    };

    ChannelOpened::Sending(socket.send(frame))
}

pub enum ChannelOpened<S>
where
    S: Sink,
{
    Sending(Send<S>),
    Receiving(Should<S>),
}

impl<S, E> Future for ChannelOpened<S>
where
    S: Stream<Item = Frame, Error = E> + Sink<SinkItem = Frame, SinkError = E>,
    E: From<Error>,
{
    type Item = S;
    type Error = E;

    fn poll(&mut self) -> Poll<S, S::Error> {
        use self::ChannelOpened::*;

        *self = match self {
            &mut Sending(ref mut sending) => {
                let socket = try_ready!(sending.poll());
                Receiving(Should::new(socket))
            }
            &mut Receiving(ref mut socket) => {
                let frame = try_stream_ready!(socket.as_mut().poll());
                match frame
                    .method()
                    .and_then(|m| m.channel())
                    .and_then(|c| c.open_ok())
                {
                    Some(_open_ok) => return Ok(Async::Ready(socket.take())),
                    None => {
                        return Err(E::from(Error::from(ErrorKind::UnexpectedFrame(
                            "OpenOk".into(),
                            frame.clone(),
                        ))))
                    }
                }
            }
        };

        self.poll()
    }
}
