use futures::sync::mpsc::{unbounded, UnboundedSender, UnboundedReceiver};
use futures::sync::oneshot;
use futures::{Future, Canceled};

use bytes::Bytes;

use amqpr_codec::frame::{Frame, FramePayload, FrameHeader, FrameType};
use amqpr_codec::frame::content_header::ContentHeaderPayload;
use amqpr_codec::frame::content_body::ContentBodyPayload;
use amqpr_codec::frame::method::MethodPayload;

use methods::{exchange, queue, basic};
use super::handler::LocalChannelCommand;


#[derive(Clone, Debug)]
pub struct LocalChannelController {
    pub channel_id: u16,
    pub command_sender: UnboundedSender<LocalChannelCommand>,
}



impl LocalChannelController {
    /// You must call this function before call another function.
    pub fn init(self) -> Box<Future<Item = LocalChannelController, Error = Canceled>> {
        let (sender, receiver) = oneshot::channel();
        self.command_sender
            .unbounded_send(LocalChannelCommand::OpenChannel(sender))
            .expect("Fail to send item");
        Box::new(receiver.map(move |_| self))
    }


    pub fn arrive_frame(&self, frame: Frame) {
        self.command_sender
            .unbounded_send(LocalChannelCommand::ArriveFrame(frame))
            .expect("Fail to send item");
    }


    pub fn declare_exchange(&self, args: exchange::DeclareArguments) {
        let frame = self.method_frame(exchange::declare(args));

        self.command_sender
            .unbounded_send(LocalChannelCommand::DeclareExchange(frame))
            .expect("Fail to send item");
    }


    pub fn declare_queue(&self, args: queue::DeclareArguments) -> oneshot::Receiver<String> {
        let frame = self.method_frame(queue::declare(args));
        let (sender, receiver) = oneshot::channel();

        self.command_sender
            .unbounded_send(LocalChannelCommand::DeclareQueue(frame, sender))
            .expect("Fail to send item");

        receiver
    }


    pub fn bind_queue(&self, args: queue::BindArguments) {
        let frame = self.method_frame(queue::bind(args));

        self.command_sender
            .unbounded_send(LocalChannelCommand::BindQueue(frame))
            .expect("Fail to send item");
    }


    pub fn publish(&self, args: basic::PublishArguments, bytes: Bytes) {
        let method_frame = self.method_frame(basic::publish(args));
        let content_header_frame = self.content_header_frame(60, &bytes); // class_id is 60
        let content_body_frame = self.content_body_frame(bytes);

        self.command_sender
            .unbounded_send(LocalChannelCommand::Publish(
                method_frame,
                content_header_frame,
                content_body_frame,
            ))
            .expect("Fail to send item");
    }


    pub fn consume(&self, args: basic::ConsumeArguments) -> UnboundedReceiver<Bytes> {
        let frame = self.method_frame(basic::consume(args));

        let (sender, receiver) = unbounded();

        self.command_sender
            .unbounded_send(LocalChannelCommand::Consume(frame, sender))
            .expect("Fail to send item");

        receiver
    }


    // Internal use
    fn method_frame(&self, method: MethodPayload) -> Frame {
        let header = FrameHeader {
            frame_type: FrameType::Method,
            channel: self.channel_id,
        };

        let frame = Frame {
            header: header,
            payload: FramePayload::Method(method),
        };

        frame
    }


    // Internal use
    fn content_header_frame(&self, class_id: u16, bytes: &Bytes) -> Frame {
        let header = FrameHeader {
            frame_type: FrameType::ContentHeader,
            channel: self.channel_id,
        };

        let payload = ContentHeaderPayload {
            class_id: class_id,
            body_size: bytes.len() as u64,
            property_flags: 1,
        };

        Frame {
            header: header,
            payload: FramePayload::ContentHeader(payload),
        }
    }


    // TODO : We should handle frame deliminating
    // Internal use
    fn content_body_frame(&self, bytes: Bytes) -> Frame {
        let header = FrameHeader {
            frame_type: FrameType::ContentBody,
            channel: self.channel_id,
        };

        let payload = ContentBodyPayload { bytes: bytes };

        Frame {
            header: header,
            payload: FramePayload::ContentBody(payload),
        }
    }
}
