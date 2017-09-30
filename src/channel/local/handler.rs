use futures::sync::mpsc::{unbounded, UnboundedSender};
use futures::sync::oneshot;
use futures::Stream;
use tokio_core::reactor::Handle;

use bytes::Bytes;

use amqpr_codec::frame::{Frame, FramePayload, FrameHeader, FrameType};
use amqpr_codec::frame::method::MethodPayload;
use amqpr_codec::frame::content_header::ContentHeaderPayload;
use amqpr_codec::frame::content_body::ContentBodyPayload;
use amqpr_codec::methods::args::*;

use socket::SocketController;
use channel::LocalChannelController;



#[derive(Debug)]
pub struct LocalChannel {
    pub channel_id: u16,
    socket_controller: SocketController,
    declare_queue_notify: Option<oneshot::Sender<String>>,
    open_channel_notify: Option<oneshot::Sender<()>>,
    consume_sink: Option<UnboundedSender<Bytes>>,
}


#[derive(Debug)]
pub enum LocalChannelCommand {
    ArriveFrame(Frame),
    DeclareExchange(Frame),
    DeclareQueue(Frame, oneshot::Sender<String>),
    BindQueue(Frame),
    Publish(Frame, Frame, Frame),
    Consume(Frame, UnboundedSender<Bytes>),
    OpenChannel(oneshot::Sender<()>),
}



impl LocalChannel {
    // This is not async.
    pub fn send_frame(&self, frame: Frame) {
        self.socket_controller.send_frame(frame);
    }


    fn handle(&mut self, command: LocalChannelCommand) {
        match command {
            LocalChannelCommand::ArriveFrame(frame) => self.handle_frame(frame),
            LocalChannelCommand::DeclareExchange(frame) => {
                self.send_frame(frame);
            }
            LocalChannelCommand::DeclareQueue(frame, sender) => {
                self.send_frame(frame);
                self.declare_queue_notify = Some(sender);
            }
            LocalChannelCommand::BindQueue(frame) => {
                self.send_frame(frame);
            }
            LocalChannelCommand::Publish(method_frame, header_frame, body_frame) => {
                self.send_frame(method_frame);
                self.send_frame(header_frame);
                self.send_frame(body_frame);
            }
            LocalChannelCommand::Consume(frame, sender) => {
                self.consume_sink = Some(sender);
                self.send_frame(frame);
            }
            LocalChannelCommand::OpenChannel(sender) => {
                self.open_channel(sender);
            }
        }
    }


    fn handle_frame(&mut self, frame: Frame) {
        match frame.payload {
            FramePayload::Method(method) => self.handle_method(method),
            FramePayload::ContentHeader(header) => self.handle_content_header(header),
            FramePayload::ContentBody(body) => self.handle_content_body(body),
            FramePayload::Heartbeat => unreachable!(),
        }
    }


    fn handle_method(&mut self, method: MethodPayload) {
        match (method.class_id, method.method_id) {
            (20, 11) => {
                self.open_channel_notify
                    .take()
                    .unwrap()
                    .send(())
                    .expect("Fail to send notify");
            }
            (20, 40) => {
                info!("Channel is closed.\n{:?}", method);
            }
            (40, 11) => {
                debug!("Declare exchange ok.\n{:?}", method);
            }
            (50, 11) => {
                debug!("Declare queue ok.\n{:?}", method);
                let queue_name = match method.arguments[0] {
                    Argument::ShortString(ShortString(ref s)) => s.clone(),
                    _ => unreachable!(),
                };
                self.declare_queue_notify
                    .take()
                    .unwrap()
                    .send(queue_name)
                    .expect("Fail to send notify");
            }
            (50, 21) => {
                debug!("Bind queue ok.\n{:?}", method);
            }
            (60, 21) => {
                debug!("Consume ok.\n{:?}", method);
            }
            (60, 60) => {
                debug!("Deliver.\n{:?}", method);
            }
            _ => panic!(),
        }
    }


    fn send_method(&mut self, payload: MethodPayload) {
        let header = FrameHeader {
            frame_type: FrameType::Method,
            channel: self.channel_id,
        };
        let frame = Frame {
            header: header,
            payload: FramePayload::Method(payload),
        };
        self.send_frame(frame);
    }


    // TODO
    fn handle_content_header(&mut self, header: ContentHeaderPayload) {
        debug!("Get content header : {:?}", header);
    }


    fn handle_content_body(&mut self, body: ContentBodyPayload) {
        debug!("Get content body : {:?}", body);

        if let Some(ref consume_sink) = self.consume_sink {
            consume_sink.unbounded_send(body.bytes).expect("Fail to send content body");
        } else {
            unreachable!();
        }
    }


    fn open_channel(&mut self, sender: oneshot::Sender<()>) {
        let open_channel_method = MethodPayload {
            class_id: 20,
            method_id: 10,
            arguments: Arguments(vec![Argument::ShortString(ShortString("".into()))]),
        };
        self.send_method(open_channel_method);

        self.open_channel_notify = Some(sender);
    }
}



pub fn create_channel(channel_id: u16,
                      socket_controller: SocketController,
                      handle: &Handle)
                      -> LocalChannelController {

    let (command_sender, command_receiver) = unbounded();

    let controller = LocalChannelController {
        channel_id: channel_id,
        command_sender: command_sender,
    };

    let mut channel = LocalChannel {
        channel_id: channel_id,
        socket_controller: socket_controller,
        declare_queue_notify: None,
        open_channel_notify: None,
        consume_sink: None,
    };

    handle.spawn(command_receiver.for_each(move |command| {
        channel.handle(command);
        Ok(())
    }));

    controller
}
