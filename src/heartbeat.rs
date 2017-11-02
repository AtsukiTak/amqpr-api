use futures::sink::{Sink, Send};

use amqpr_codec::frame::{Frame, FrameHeader, FramePayload};


pub fn send_heartbeat<S>(sink: S) -> SentHeartBeat<S>
where
    S: Sink<SinkItem = Frame>,
{
    let frame = Frame {
        header: FrameHeader { channel: 0 },
        payload: FramePayload::Heartbeat,
    };
    sink.send(frame)
}


type SentHeartBeat<S> = Send<S>;
