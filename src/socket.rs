use tokio_core::net::TcpStream;
use tokio_core::reactor::Handle;
use tokio_io::io::{write_all, WriteAll};
use tokio_io::AsyncRead;
use futures::{Future, Stream, Canceled};
use futures::sink::Sink;
use futures::sync::mpsc::{unbounded, UnboundedSender};

use amqpr_codec::frame::{Codec, Frame};

use std::net::SocketAddr;
use std::collections::HashMap;

use channel::{GlobalChannelController, LocalChannelController, global_channel};


#[derive(Debug)]
pub(crate) enum SocketCommand {
    ArriveFrame(Frame),
    SendFrame(Frame),
    AddChannel(LocalChannelController),
}



#[derive(Clone, Debug)]
pub(crate) struct SocketController(UnboundedSender<SocketCommand>);


#[derive(Debug)]
struct SocketHandler {
    global_channel_controller: GlobalChannelController,

    local_channels: HashMap<u16, LocalChannelController>,

    frame_sender: UnboundedSender<Frame>,
}


pub fn open(
    addr: &SocketAddr,
    handle: &Handle,
    user: String,
    pass: String,
) -> Box<Future<Item = GlobalChannelController, Error = Canceled>> {

    // Declare SocketController
    let (socket_command_sender, socket_command_receiver) = unbounded();
    let socket_controller = SocketController(socket_command_sender);

    // Declare GlobalChannel
    let (global_channel_controller, finish_handshake_notify) =
        global_channel(socket_controller.clone(), handle, user, pass);

    // Declare SocketHandler
    let (frame_sender, frame_receiver) = unbounded();
    let mut socket_handler = SocketHandler {
        global_channel_controller: global_channel_controller.clone(),
        local_channels: HashMap::new(),
        frame_sender: frame_sender,
    };

    // Run SocketHandler
    handle.spawn(socket_command_receiver.for_each(move |command| {
        socket_handler.handle_command(command);
        Ok(())
    }));


    let handle2 = handle.clone();

    // Open socket
    let future = TcpStream::connect(addr, handle)
        .and_then(send_protocol_header)
        .map_err(|err| {
            error!("error : {:?}", err);
            ()
        })
        .map(move |(socket, _)| {

            // create Stream and Sink
            let (sink, stream) = socket.framed(Codec).split();

            // Send frame to network.
            handle2.spawn_fn(move || {
                sink.sink_map_err(|e| {
                    error!("error : {:?}", e);
                    ()
                }).send_all(frame_receiver)
                    .map(|_| ())
            });

            // Receive items from Rabbitmq then send it to handler.
            handle2.spawn_fn(move || {
                stream
                    .map_err(|err| {
                        error!("error : {:?}", err);
                        ()
                    })
                    .for_each(move |frame| {
                        socket_controller.arrive_frame(frame);
                        Ok(())
                    })
            });
        });

    handle.spawn(future);

    Box::new(finish_handshake_notify.map(
        move |_| global_channel_controller,
    ))
}


impl SocketHandler {
    /// Most important function of `ConnectionManager`.
    /// Handle each message.
    fn handle_command(&mut self, command: SocketCommand) {
        match command {
            SocketCommand::AddChannel(controller) => {
                self.local_channels.insert(
                    controller.channel_id,
                    controller,
                );
            }
            SocketCommand::SendFrame(frame) => {
                (&self.frame_sender).unbounded_send(frame).expect(
                    "Fail to send frame",
                );
            }
            SocketCommand::ArriveFrame(frame) => {
                debug!("New frame is arrived\n{:?}", frame);
                if frame.header.channel == 0_u16 {
                    self.global_channel_controller.arrive_frame(frame);
                } else if let Some(controller) = self.local_channels.get(&frame.header.channel) {
                    controller.arrive_frame(frame);
                } else {
                    unreachable!();
                }
            }
        }
    }
}



impl SocketController {
    fn arrive_frame(&self, frame: Frame) {
        (&self.0)
            .unbounded_send(SocketCommand::ArriveFrame(frame))
            .expect("Fail to send SocketCommand");
    }

    pub(crate) fn send_frame(&self, frame: Frame) {
        (&self.0)
            .unbounded_send(SocketCommand::SendFrame(frame))
            .expect("Fail to send SocketCommand");
    }

    pub(crate) fn add_local_channel(&self, controller: LocalChannelController) {
        (&self.0)
            .unbounded_send(SocketCommand::AddChannel(controller))
            .expect("Fail to send SocketCommand");
    }
}





/// This is same with
/// | "A" | "M" | "Q" | "P" | 0 | 0 | 9 | 1 |
const PROTOCOL_HEADER: [u8; 8] = [65, 77, 81, 80, 0, 0, 9, 1];

/// Just send protocol header which is maybe accepted.
/// Not doing negotiation. If it fails to negotiate, this function returns error.
fn send_protocol_header(socket: TcpStream) -> WriteAll<TcpStream, [u8; 8]> {
    debug!("Send protocol header");
    write_all(socket, PROTOCOL_HEADER)
}
