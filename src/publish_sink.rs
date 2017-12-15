//! Convenient module to publish item.
use futures::{Sink, Future, Poll, StartSend, Async, AsyncSink};

use amqpr_codec::Frame;

use basic::publish::{publish, Published, PublishItem};
use common::Should;




/// Returns `BroadcastSink` which is `Sink` of `PublishItem`.
///
/// You may also need to have a look at `PublishItem` document.
pub fn publish_sink<S, E>(channel: u16, socket: S) -> BroadcastSink<S>
where
    S: Sink<SinkItem = Frame>,
{
    BroadcastSink {
        channel: channel,
        state: PublishState::Waiting(Should::new(socket)),
    }
}


/// A outbound endpoint to publish data.
pub struct BroadcastSink<S>
where
    S: Sink<SinkItem = Frame>,
{
    channel: u16,
    state: PublishState<S>,
}


enum PublishState<S>
where
    S: Sink<SinkItem = Frame>,
{
    Processing(Published<S>),
    Waiting(Should<S>),
}


impl<S> Sink for BroadcastSink<S>
where
    S: Sink<SinkItem = Frame>,
{
    type SinkItem = PublishItem;
    type SinkError = S::SinkError;

    fn start_send(&mut self, item: PublishItem) -> StartSend<PublishItem, Self::SinkError> {

        if let Async::NotReady = self.poll_complete()? {
            return Ok(AsyncSink::NotReady(item));
        }

        use self::PublishState::*;
        self.state = match &mut self.state {
            &mut Processing(ref mut _published) => unreachable!(),
            &mut Waiting(ref mut sink) => {
                let sink = sink.take();
                let published = publish(sink, self.channel, item);
                Processing(published)
            }
        };

        Ok(AsyncSink::Ready)
    }


    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        use self::PublishState::*;

        self.state = match &mut self.state {
            &mut Processing(ref mut processing) => {
                let sink = try_ready!(processing.poll());
                Waiting(Should::new(sink))
            }
            &mut Waiting(_) => return Ok(Async::Ready(())),
        };

        self.poll_complete()
    }
}
