use futures::sync::mpsc::unbounded;
use futures::sync::oneshot;
use futures::Stream;
use tokio_core::reactor::{Handle, Interval};

use amqpr_codec::frame::{Frame, FramePayload, FrameHeader, FrameType};
use amqpr_codec::frame::method::MethodPayload;

use std::time::Duration;

use socket::SocketController;
use methods::connection::{start_ok, tune_ok, open};
use channel::{LocalChannelController, GlobalChannelController, local};


const HEARTBEAT_INTERVAL_SEC: u64 = 60;



#[derive(Debug)]
pub struct GlobalChannel {
    socket_controller: SocketController,
    finish_handshake_notify: Option<oneshot::Sender<()>>,
    open_channel_notify: Option<oneshot::Sender<()>>,

    // Used when open new local channel.
    // See `open_channel` function.
    handle: Handle,
    user: String,
    pass: String,
}



#[derive(Debug)]
pub enum GlobalChannelCommand {
    ArriveFrame(Frame),
    DeclareLocalChannel(u16, oneshot::Sender<LocalChannelController>),
    SendHeartbeatFrame,
}



impl GlobalChannel {
    fn handle(&mut self, command: GlobalChannelCommand) {
        match command {
            GlobalChannelCommand::ArriveFrame(frame) => self.handle_frame(frame),
            GlobalChannelCommand::DeclareLocalChannel(channel_id, sender) => {
                let controller = self.declare_local_channel(channel_id);
                sender.send(controller).expect("Fail to send local channel controlelr");
            }
            GlobalChannelCommand::SendHeartbeatFrame => {
                self.send_heartbeat_frame();
            }
        }
    }


    fn handle_frame(&mut self, frame: Frame) {
        match frame.payload {
            FramePayload::Method(method) => self.handle_method(method),
            FramePayload::Heartbeat => debug!("Heartbeat frame is received"),
            _ => unreachable!(),
        }
    }


    fn handle_method(&mut self, method: MethodPayload) {
        match (method.class_id, method.method_id) {
            (10, 10) => self.send_method(start_ok(&method, self.user.as_str(), self.pass.as_str())),
            (10, 30) => {
                self.send_method(tune_ok(&method));
                self.send_method(open("/"));
            }
            (10, 41) => {
                self.finish_handshake_notify
                    .take()
                    .unwrap()
                    .send(())
                    .expect("Fail to send finish handshake notify");
            }
            (10, 50) => {
                info!("Connection is closed\n{:?}", method);
            }
            (20, 11) => {
                self.open_channel_notify
                    .take()
                    .unwrap()
                    .send(())
                    .expect("Fail to send open channel notify");
            }
            (20, 40) => {
                info!("Channel is closed\n{:?}", method);
            }
            _ => unreachable!(),
        }
    }

    fn send_method(&self, payload: MethodPayload) {
        let header = FrameHeader {
            frame_type: FrameType::Method,
            channel: 0_u16,
        };
        let frame = Frame {
            header: header,
            payload: FramePayload::Method(payload),
        };
        self.send_frame(frame);
    }


    fn send_heartbeat_frame(&self) {
        let header = FrameHeader {
            frame_type: FrameType::Heartbeat,
            channel: 0_u16,
        };
        let frame = Frame {
            header: header,
            payload: FramePayload::Heartbeat,
        };
        self.send_frame(frame);
    }


    fn send_frame(&self, frame: Frame) {
        self.socket_controller.send_frame(frame);
    }


    fn declare_local_channel(&mut self, channel_id: u16) -> LocalChannelController {
        let channel_controller = local::handler::create_channel(channel_id,
                                                                self.socket_controller.clone(),
                                                                &self.handle);

        // Just add new local channel to local_channels field of SocketHandler.
        self.socket_controller.add_local_channel(channel_controller.clone());

        channel_controller
    }
}



type FinishHandshakeNotify = oneshot::Receiver<()>;

pub fn global_channel(socket_controller: SocketController,
                      handle: &Handle,
                      user: String,
                      pass: String)
                      -> (GlobalChannelController, FinishHandshakeNotify) {

    let (sender, receiver) = oneshot::channel();

    let mut global_channel = GlobalChannel {
        socket_controller: socket_controller,
        finish_handshake_notify: Some(sender),
        open_channel_notify: None,
        handle: handle.clone(),
        user: user,
        pass: pass,
    };

    let (command_sender, command_receiver) = unbounded();

    let controller = GlobalChannelController(command_sender);
    handle.spawn(command_receiver.for_each(move |command| {
        global_channel.handle(command);
        Ok(())
    }));

    send_hardbeat(handle, controller.clone());

    (controller, receiver)
}


fn send_hardbeat(handle: &Handle, controller: GlobalChannelController) {
    let heartbeat_interval = Duration::new(HEARTBEAT_INTERVAL_SEC, 0);
    handle.spawn(
        Interval::new(heartbeat_interval, handle)
            .expect("Fail to create internal stream")
            .map_err(|e| panic!("Error occured during process interval stream : {:?}", e))
            .for_each(move |()| Ok(controller.send_heartbeat()))
    );
}
