use amqpr_codec::{Frame, FrameHeader, FramePayload};
use amqpr_codec::method::MethodPayload;
use amqpr_codec::method::channel::{ChannelClass, OpenMethod};

use futures::{Future, Stream, Sink};

use errors::*;


pub type ChannelOpenFuture<S, E> = Box<Future<Item = (Frame, S), Error = E>>;


/// Open new channel with given channel id.
///
/// Returned future consists of `Frame` and `S` but for now, `Frame` is meanless
/// because it has nothing.
pub fn open_channel<S, E>(socket: S, channel_id: u16) -> ChannelOpenFuture<S, E>
where
    S: Stream<Item = Frame, Error = E> + Sink<SinkItem = Frame, SinkError = E> + 'static,
    E: From<Error> + 'static,
{
    let open = OpenMethod { reserved1: "".into() };
    let frame = Frame {
        header: FrameHeader { channel: channel_id },
        payload: FramePayload::Method(MethodPayload::Channel(ChannelClass::Open(open))),
    };

    Box::new(
        socket
            .send(frame)
            .and_then(move |socket| socket.into_future().map_err(|(err, _s)| err))
            .and_then(|(frame_opt, socket)| match frame_opt {
                None => Err(E::from(Error::from(ErrorKind::UnexpectedConnectionClose))),
                Some(open_ok) => Ok((open_ok, socket)),
            }),
    ) as ChannelOpenFuture<S, E>
}
