use amqpr_codec::Frame;
use futures::{Stream, Sink};


pub struct Should<T>(Option<T>);

impl<T> Should<T> {
    pub fn new(item: T) -> Should<T> {
        Should(Some(item))
    }

    pub fn as_ref(&self) -> &T {
        self.0.as_ref().expect("You never use item which is already taken")
    }

    pub fn as_mut(&mut self) -> &mut T {
        self.0.as_mut().expect("You never use item which is already taken")
    }

    pub fn take(&mut self) -> T {
        self.0.take().expect("You never use item which is already taken")
    }
}




pub fn send_and_receive(socket: ::AmqpSocket, item: Frame) -> SendAndReceive {
    SendAndReceive::Sending(socket.send(item))
}


pub enum SendAndReceive {
    Sending(::futures::sink::Send<::AmqpSocket>),
    Receiving(Should<::AmqpSocket>),
}



impl ::futures::Future for SendAndReceive {
    type Item = (Frame, ::AmqpSocket);
    type Error = ::errors::Error;

    fn poll(&mut self) -> ::futures::Poll<Self::Item, Self::Error> {
        use self::SendAndReceive::*;
        *self = match self {
            &mut Sending(ref mut sending) => Receiving(Should::new(try_ready!(sending))),
            &mut Receiving(ref mut socket) => {
                let item = poll_item!(socket.as_mut());
                return Ok(::futures::Async::Ready((item, socket.take())));
            }
        };
        self.poll()
    }
}
