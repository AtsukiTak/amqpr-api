use bytes::Bytes;

use amqpr_codec::Frame;

use futures::{Future, Stream, Poll, Async};

use std::borrow::Borrow;

use common::Should;
use errors::*;


pub fn receive_delivered<S>(stream: S) -> Delivered<S>
where
    S: Stream,
    S::Item: Borrow<Frame>,
    S::Error: From<Error>,
{
    Delivered::ReceivingDeliverMethod(Should::new(stream))
}



// Delivered struct {{{
pub enum Delivered<S> {
    ReceivingDeliverMethod(Should<S>),
    ReceivingContentHeader(Should<S>),
    ReceivingContentBody(Should<S>),
}


impl<S> Future for Delivered<S>
where
    S: Stream,
    S::Item: Borrow<Frame>,
    S::Error: From<Error>,
{
    type Item = (Bytes, S);
    type Error = S::Error;

    fn poll(&mut self) -> Poll<(Bytes, S), S::Error> {

        use self::Delivered::*;
        *self = match self {

            // Receive deliver method frame. Ignore another frame.
            &mut ReceivingDeliverMethod(ref mut socket) => {
                let deliver = loop {
                    let frame = poll_item!(socket.as_mut());

                    let deliver = frame.borrow().method().and_then(|m| m.basic()).and_then(
                        |c| {
                            c.deliver()
                        },
                    );
                    match deliver {
                        Some(del) => break del.clone(),
                        None => continue,
                    }
                };
                info!("Deliver method is received : {:?}", deliver);
                ReceivingContentHeader(Should::new(socket.take()))
            }

            // Ignore another frame.
            &mut ReceivingContentHeader(ref mut socket) => {
                let ch = loop {
                    let frame = poll_item!(socket.as_mut());
                    match frame.borrow().content_header() {
                        Some(ch) => break ch.clone(),
                        None => continue,
                    }
                };
                info!("Content header is received : {:?}", ch);

                ReceivingContentBody(Should::new(socket.take()))
            }

            // Ignore another frame.
            &mut ReceivingContentBody(ref mut socket) => {
                let cb = loop {
                    let frame = poll_item!(socket.as_mut());
                    match frame.borrow().content_body() {
                        Some(cb) => break cb.clone(),
                        None => continue,
                    }
                };

                info!("Content body is received : {:?}", cb);
                return Ok(Async::Ready((cb.bytes.clone(), socket.take())));
            }
        };

        self.poll()
    }
}
// }}}
