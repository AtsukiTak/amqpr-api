//! Convenient module to subscribe item.

use futures::{Future, Stream, Sink, Poll, Async};

use amqpr_codec::Frame;

use basic::consume::{start_consume, ConsumeStarted};
use basic::deliver::{get_delivered, Delivered, DeliveredItem};
use errors::Error;

pub use basic::consume::StartConsumeOption;



/// Returns `SubscribeStream` which is `Stream` of `DeliveredItem`.
/// Basically, this function send `Consume` message to AMQO server first, and then
/// wait for each items. This uses `get_delivered` method internally. If you want to know more detail
/// please have a look at `get_delivered` function document.
///
/// We skips an item being not considered as subscribe item such as `Heartbeat` or `Error`.
/// So we recommend that one local channel has only one subscribe stream.
///
/// # Notice
/// Here is the list of default options we set in `Consume` method.
///
/// - no-local: false
/// - no-ack: true
/// - exclusive: true
/// - no-wait: true
pub fn subscribe_stream<S, E>(
    ch_id: u16,
    socket: S,
    option: StartConsumeOption,
) -> SubscribeStream<S, E>
where
    S: Stream<Item = Frame, Error = E> + Sink<SinkItem = Frame, SinkError = E>,
    E: From<Error>,
{
    let consume_started = start_consume(ch_id, socket, option);
    let sub_stream = SubscribeStream::SendingConsumeMethod(consume_started);

    sub_stream
}






/// Stream of subscribed item from AMQP server.
/// This stream is based on `no_ack` consume because of performance.
/// But that may cause decreasing of reliability.
/// If you want reliability rather than performance, you should use `subscribe_stream_ack`
/// function (after we implement that...).
pub enum SubscribeStream<S, E>
where
    S: Stream<Item = Frame, Error = E> + Sink<SinkItem = Frame, SinkError = E>,
{
    SendingConsumeMethod(ConsumeStarted<S>),
    ReceivingDeliverd(Delivered<S>),
}


impl<S, E> Stream for SubscribeStream<S, E>
where
    S: Stream<Item = Frame, Error = E>
        + Sink<SinkItem = Frame, SinkError = E>,
    E: From<Error>,
{
    type Item = DeliveredItem;
    type Error = E;

    fn poll(&mut self) -> Poll<Option<DeliveredItem>, Self::Error> {
        use self::SubscribeStream::*;

        let (item, socket) = match self {
            &mut SendingConsumeMethod(ref mut fut) => {
                let socket = try_ready!(fut.poll());
                (None, socket)
            }
            &mut ReceivingDeliverd(ref mut del) => {
                let (item, socket) = try_ready!(del.poll());
                (Some(item), socket)
            }
        };

        // Start to receive new delivered item.
        let delivered = get_delivered(socket);
        *self = ReceivingDeliverd(delivered);

        match item {
            Some(bytes) => Ok(Async::Ready(Some(bytes))),
            None => self.poll(),
        }
    }
}
