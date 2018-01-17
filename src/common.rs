#[derive(Debug, Clone)]
pub struct Should<T>(Option<T>);

impl<T> Should<T> {
    pub fn new(item: T) -> Should<T> {
        Should(Some(item))
    }

    pub fn as_ref(&self) -> &T {
        self.0
            .as_ref()
            .expect("You never use item which is already taken")
    }

    pub fn as_mut(&mut self) -> &mut T {
        self.0
            .as_mut()
            .expect("You never use item which is already taken")
    }

    pub fn take(&mut self) -> T {
        self.0
            .take()
            .expect("You never use item which is already taken")
    }
}

/*
pub fn send_and_receive<In, Out, F>(
    msg: Out::SinkItem,
    income: In,
    outcome: Out,
    f: F,
) -> SendAndReceive<In, Out, F>
where
    Out: Sink,
{
    SendAndReceive {
        state: SendAndReceiveState::Sending(Should::new(income), outcome.send(msg)),
        f: f,
    }
}



pub struct SendAndReceive<In, Out, F>
where
    Out: Sink,
{
    state: SendAndReceiveState<In, Out>,
    f: F,
}


pub enum SendAndReceiveState<In, Out>
where
    Out: Sink,
{
    Sending(Should<In>, Send<Out>),
    Receiving(Should<In>, Should<Out>),
}



impl<In, Out, F, E> Future for SendAndReceive<In, Out, F>
where
    In: Stream<Error = E>,
    In::Item: Borrow<Frame>,
    Out: Sink<SinkError = E>,
    F: FnMut(&Frame) -> bool,
    E: From<Error>,
{
    type Item = (In::Item, In, Out);
    type Error = E;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {

        use self::SendAndReceiveState::*;
        self.state = match &mut self.state {
            &mut Sending(ref mut income, ref mut sending) => {
                let out = try_ready!(sending.poll());
                Receiving(Should::new(income.take()), Should::new(out))
            }
            &mut Receiving(ref mut income, ref mut outcome) => {
                let item = loop {
                    let item = poll_item!(income.as_mut());
                    if (self.f)(item.borrow()) {
                        break item;
                    }
                };
                return Ok(Async::Ready((item, income.take(), outcome.take())));
            }
        };
        self.poll()
    }
}
*/
