use amqpr_codec::Frame;
use amqpr_codec::content_header::ContentHeaderPayload;
use amqpr_codec::content_body::ContentBodyPayload;
use amqpr_codec::frame::method::basic::DeliverMethod;

use futures::{Future, Stream, Poll, Async};

use common::Should;
use errors::*;


/// Get `DeliveredItem` from given stream.
/// `DeliveredItem` consists of three things; `DeliverMethod`, `ContentHeaderPayload` and
/// `ContentBodyPayload`.
///
/// # Notice
/// Maybe it is useful to use `subscribe_stream`. That function returns `Stream` of
/// `DeliveredItem`.
///
/// # Error
/// `Delivered` future might be `Error` when `stream: S` yields unexpected frame.
pub fn get_delivered<S>(stream: S) -> Delivered<S>
where
    S: Stream<Item = Frame>,
    S::Error: From<Error>,
{
    Delivered::ReceivingDeliverMethod(Should::new(stream))
}


/// The value in `Future` being returned by `get_delivered` function.
#[derive(Debug, Clone)]
pub struct DeliveredItem {
    pub meta: DeliverMethod,
    pub header: ContentHeaderPayload,
    pub body: ContentBodyPayload,
}


// Delivered struct {{{
pub enum Delivered<S> {
    ReceivingDeliverMethod(Should<S>),
    ReceivingContentHeader(Should<S>, Should<DeliverMethod>),
    ReceivingContentBody(Should<S>, Should<(DeliverMethod, ContentHeaderPayload)>),
}


impl<S> Future for Delivered<S>
where
    S: Stream<Item = Frame>,
    S::Error: From<Error>,
{
    type Item = (DeliveredItem, S);
    type Error = S::Error;

    fn poll(&mut self) -> Poll<(DeliveredItem, S), S::Error> {

        use self::Delivered::*;
        *self = match self {

            &mut ReceivingDeliverMethod(ref mut socket) => {
                let frame = try_stream_ready!(socket.as_mut().poll());

                let is_deliver = frame.method().and_then(|m| m.basic()).and_then(
                    |c| c.deliver(),
                );
                let deliver = match is_deliver {
                    Some(del) => del.clone(),
                    None => {
                        return Err(S::Error::from(Error::from(
                            ErrorKind::UnexpectedFrame("Deliver".into(), frame.clone()),
                        )))
                    }
                };
                info!("Deliver method is received : {:?}", deliver);
                ReceivingContentHeader(Should::new(socket.take()), Should::new(deliver))
            }

            &mut ReceivingContentHeader(ref mut socket, ref mut deliver) => {
                let frame = try_stream_ready!(socket.as_mut().poll());
                let header = match frame.content_header() {
                    Some(ch) => ch.clone(),
                    None => {
                        return Err(S::Error::from(Error::from(
                            ErrorKind::UnexpectedFrame("Deliver".into(), frame.clone()),
                        )))
                    }
                };
                info!("Content header is received : {:?}", header);

                ReceivingContentBody(
                    Should::new(socket.take()),
                    Should::new((deliver.take(), header)),
                )
            }

            &mut ReceivingContentBody(ref mut socket, ref mut piece) => {
                let frame = try_stream_ready!(socket.as_mut().poll());
                let body = match frame.content_body() {
                    Some(cb) => cb.clone(),
                    None => {
                        return Err(S::Error::from(Error::from(
                            ErrorKind::UnexpectedFrame("Deliver".into(), frame.clone()),
                        )))
                    }
                };

                info!("Content body is received : {:?}", body);
                let (meta, header) = piece.take();
                let item = DeliveredItem {
                    meta: meta,
                    header: header,
                    body: body,
                };
                return Ok(Async::Ready((item, socket.take())));
            }
        };

        self.poll()
    }
}
// }}}
