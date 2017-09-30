use futures::sync::mpsc::UnboundedSender;
use futures::Future;
use futures::sync::oneshot;

use amqpr_codec::frame::Frame;

use channel::LocalChannelController;
use super::handler::GlobalChannelCommand;


/// This is sendable across thread.
#[derive(Clone, Debug)]
pub struct GlobalChannelController(pub UnboundedSender<GlobalChannelCommand>);



impl GlobalChannelController {
    pub fn arrive_frame(&self, frame: Frame) {
        self.0
            .unbounded_send(GlobalChannelCommand::ArriveFrame(frame))
            .expect("Fail to send command");
    }



    // Just declare new local channel.
    // It will be registered in SocketManager but not AMQP server
    // so you MUST call `LocalChannelController::init()` function
    // before using that channel.
    pub fn declare_local_channel(self,
                                 channel_id: u16)
                                 -> Box<Future<Item = (GlobalChannelController,
                                                       LocalChannelController),
                                               Error = oneshot::Canceled>> {

        let (sender, receiver) = oneshot::channel();

        self.0
            .unbounded_send(GlobalChannelCommand::DeclareLocalChannel(channel_id, sender))
            .expect("Fail to send command");

        Box::new(receiver.map(move |local| (self, local)))
    }


    pub fn send_heartbeat(&self) {
        self.0
            .unbounded_send(GlobalChannelCommand::SendHeartbeatFrame)
            .expect("Fail to send command")
    }
}
