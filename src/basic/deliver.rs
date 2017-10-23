use bytes::Bytes;

use futures::{Future, Stream, Poll, Async};

use AmqpSocket;
use common::Should;
use errors::*;


pub fn receive_delivered(socket: AmqpSocket) -> Delivered {
    Delivered::ReceivingDeliverMethod(Should::new(socket))
}



// Delivered struct {{{
pub enum Delivered {
    ReceivingDeliverMethod(Should<AmqpSocket>),
    ReceivingContentHeader(Should<AmqpSocket>),
    ReceivingContentBody(Should<AmqpSocket>),
}


impl Future for Delivered {
    type Item = (Bytes, AmqpSocket);
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {

        use self::Delivered::*;
        *self = match self {
            &mut ReceivingDeliverMethod(ref mut socket) => {
                let frame = poll_item!(socket.as_mut());
                info!("Deliver method is received : {:?}", frame);
                let _deliver = frame
                    .method()
                    .and_then(|m| m.basic())
                    .and_then(|c| c.deliver())
                    .ok_or(Error::from(ErrorKind::UnexpectedFrame))?;
                ReceivingContentHeader(Should::new(socket.take()))
            }

            &mut ReceivingContentHeader(ref mut socket) => {
                let frame = poll_item!(socket.as_mut());
                info!("Content header is received : {:?}", frame);
                frame.content_header().ok_or(Error::from(
                    ErrorKind::UnexpectedFrame,
                ))?;
                ReceivingContentBody(Should::new(socket.take()))
            }

            &mut ReceivingContentBody(ref mut socket) => {
                let frame = poll_item!(socket.as_mut());
                info!("Content body is received : {:?}", frame);
                let bytes = frame
                    .content_body()
                    .ok_or(Error::from(ErrorKind::UnexpectedFrame))?
                    .bytes.clone();
                return Ok(Async::Ready((bytes, socket.take())));
            }
        };

        self.poll()
    }
}
// }}}
