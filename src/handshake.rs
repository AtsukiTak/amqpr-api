//! # Connection handshake
//!
//! ```text
//! Client                               AMQP Server
//!    +                                        +
//!    |     Protocol header (not framed)       |
//!    +--------------------------------------> +
//!    |                                        |
//!    |     Start method                       |
//!    | <--------------------------------------+
//!    |                                        |
//!    |     Start-ok method                    |
//!    +--------------------------------------> +
//!    |                                        |
//!    |     Secure method                      |
//!    | <--------------------------------------+
//!    |                                        |
//!    |     Secure-ok method                   |
//!    +--------------------------------------> +
//!    |                                        |
//!    |     Tune method                        |
//!    | <--------------------------------------+
//!    |                                        |
//!    |     Tune-ok method                     |
//!    +--------------------------------------> +
//!    |                                        |
//!    |     Open method                        |
//!    +--------------------------------------> +
//!    |                                        |
//!    |     Open-ok method                     |
//!    | <--------------------------------------+
//!    |                                        |
//!    |                   X                    |
//!    |                   X                    |
//!    |                   X                    |
//!    |                   X                    |
//!    |                                        |
//!    |     Close method (both peer can send)  |
//!    +--------------------------------------> |
//!    | <--------------------------------------+
//!    |                                        |
//!    |     Close-ok method                    |
//!    | <--------------------------------------+
//!    +--------------------------------------> |
//!    |                                        |
//!    |                                        |
//!    +                                        +
//! ```
//!
//!

use amqpr_codec::{FieldArgument, Frame};
use amqpr_codec::method::connection::*;
use amqpr_codec::method::MethodPayload;
use amqpr_codec::args::AmqpString;

use futures::sink::Send;
use futures::{Async, Future, Poll, Sink, Stream};

use tokio_core::net::TcpStream;
use tokio_io::io::{write_all, WriteAll};
use tokio_io::AsyncRead;

use AmqpSocket;
use common::Should;
use errors::*;

const PROTOCOL_HEADER: [u8; 8] = [b'A', b'M', b'Q', b'P', 0, 0, 9, 1];
const GLOBAL_CHANNEL_ID: u16 = 0;

pub fn start_handshake<H>(handshaker: H, socket: TcpStream) -> Handshaking<H>
where
    H: Handshaker,
{
    Handshaking {
        stage: HandshakeStage::SendingProtoHeader(write_all(socket, PROTOCOL_HEADER)),
        handshaker: handshaker,
    }
}

pub struct Handshaking<H>
where
    H: Handshaker,
{
    stage: HandshakeStage,
    handshaker: H,
}

// HandshakeStage {{{
enum HandshakeStage {
    SendingProtoHeader(WriteAll<TcpStream, [u8; 8]>),
    ReceivingStart(Should<AmqpSocket>),
    SendingStartOkOrSecureOk(Send<AmqpSocket>),
    ReceivingSecureOrTune(Should<AmqpSocket>),
    SendingTuneOk(Send<AmqpSocket>),
    SendingOpen(Send<AmqpSocket>),
    ReceivingOpenOk(Should<AmqpSocket>),
}
// }}}

// Handshaker {{{
pub trait Handshaker {
    fn reply_to_start<'a>(&mut self, &'a StartMethod) -> StartOkMethod;
    fn reply_to_secure<'a>(&mut self, &'a SecureMethod) -> SecureOkMethod;
    fn reply_to_tune<'a>(&mut self, &'a TuneMethod) -> TuneOkMethod;
    fn create_open(&mut self) -> OpenMethod;
    fn inspect_open_ok<'a>(&mut self, &'a OpenOkMethod);
}

pub struct SimpleHandshaker {
    pub user: String,
    pub pass: String,
    pub virtual_host: String,
}

impl Handshaker for SimpleHandshaker {
    fn reply_to_start<'a>(&mut self, start: &'a StartMethod) -> StartOkMethod {
        info!("Receive start method : {:?}", start);

        let properties = {
            let mut map = ::std::collections::HashMap::new();
            use self::FieldArgument::*;
            map.insert("product".into(), LongString("amqpr".into()));
            map.insert("version".into(), LongString("0.2".into()));
            map.insert("platform".into(), LongString("Rust stable".into()));
            map.insert("copyright".into(), LongString("(C) 2017 Atsuki-Tak".into()));
            map.insert("information".into(), LongString("WIP".into()));
            map
        };

        StartOkMethod {
            client_properties: properties,
            mechanism: "PLAIN".into(),
            response: AmqpString::from(format!("{}\0{}\0{}", "", self.user, self.pass)),
            locale: "en_US".into(),
        }
    }

    fn reply_to_secure<'a>(&mut self, _secure: &'a SecureMethod) -> SecureOkMethod {
        unreachable!("We use PLAIN mode for sasl, so we never receive secure method");
    }

    fn reply_to_tune<'a>(&mut self, tune: &'a TuneMethod) -> TuneOkMethod {
        info!("Receive tune method : {:?}", tune);

        TuneOkMethod {
            channel_max: tune.channel_max,
            frame_max: tune.frame_max,
            heartbeat: 60,
        }
    }

    fn create_open(&mut self) -> OpenMethod {
        OpenMethod {
            virtual_host: AmqpString::from(self.virtual_host.clone()),
            reserved1: "".into(),
            reserved2: false,
        }
    }

    fn inspect_open_ok<'a>(&mut self, open_ok: &'a OpenOkMethod) {
        info!("Receive open ok method : {:?}", open_ok);
    }
}
// }}}

// Implement Future for Handshaking {{{
impl<H> Future for Handshaking<H>
where
    H: Handshaker,
{
    type Item = AmqpSocket;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        use self::HandshakeStage::*;
        self.stage = match &mut self.stage {
            &mut SendingProtoHeader(ref mut sending_future) => {
                let (socket, _buf) = try_ready!(sending_future.poll());
                ReceivingStart(Should::new(AmqpSocket(socket.framed(::amqpr_codec::Codec))))
            }

            &mut ReceivingStart(ref mut should_socket) => {
                let frame = try_stream_ready!(should_socket.as_mut().poll());
                let start_ok = start_ok_frame(self.handshaker.reply_to_start(is_start(&frame)?));
                SendingStartOkOrSecureOk(should_socket.take().send(start_ok))
            }

            &mut SendingStartOkOrSecureOk(ref mut sending_future) => {
                let socket = try_ready!(sending_future.poll());
                ReceivingSecureOrTune(Should::new(socket))
            }

            &mut ReceivingSecureOrTune(ref mut should_socket) => {
                let frame = try_stream_ready!(should_socket.as_mut().poll());
                match is_secure_or_tune_method(&frame)? {
                    SecureOrTune::Secure(s) => {
                        let secure_ok = secure_ok_frame(self.handshaker.reply_to_secure(s));
                        SendingStartOkOrSecureOk(should_socket.take().send(secure_ok))
                    }
                    SecureOrTune::Tune(t) => {
                        let tune_ok = tune_ok_frame(self.handshaker.reply_to_tune(t));
                        SendingTuneOk(should_socket.take().send(tune_ok))
                    }
                }
            }

            &mut SendingTuneOk(ref mut sending_future) => {
                let socket = try_ready!(sending_future.poll());
                let open = open_frame(self.handshaker.create_open());
                SendingOpen(socket.send(open))
            }

            &mut SendingOpen(ref mut sending_future) => {
                let socket = try_ready!(sending_future.poll());
                ReceivingOpenOk(Should::new(socket))
            }

            &mut ReceivingOpenOk(ref mut should_socket) => {
                let frame = try_stream_ready!(should_socket.as_mut().poll());
                self.handshaker.inspect_open_ok(is_open_ok(&frame)?);
                return Ok(Async::Ready(should_socket.take()));
            }
        };

        self.poll()
    }
}

fn start_ok_frame(start_ok: StartOkMethod) -> Frame {
    connection_frame(ConnectionClass::StartOk(start_ok))
}

fn secure_ok_frame(secure_ok: SecureOkMethod) -> Frame {
    connection_frame(ConnectionClass::SecureOk(secure_ok))
}

fn tune_ok_frame(tune_ok: TuneOkMethod) -> Frame {
    connection_frame(ConnectionClass::TuneOk(tune_ok))
}

fn open_frame(open: OpenMethod) -> Frame {
    connection_frame(ConnectionClass::Open(open))
}

fn connection_frame(connection_class: ConnectionClass) -> Frame {
    Frame::new_method(
        GLOBAL_CHANNEL_ID,
        MethodPayload::Connection(connection_class),
    )
}

fn is_start(frame: &Frame) -> Result<&StartMethod, Error> {
    frame
        .method()
        .and_then(|m| m.connection())
        .and_then(|c| c.start())
        .ok_or(Error::from(ErrorKind::FailToHandshake))
}

enum SecureOrTune<'a> {
    Secure(&'a SecureMethod),
    Tune(&'a TuneMethod),
}

fn is_secure_or_tune_method<'a>(frame: &'a Frame) -> Result<SecureOrTune<'a>, Error> {
    frame
        .method()
        .and_then(|m| m.connection())
        .and_then(|c| {
            let secure_op = c.secure().map(|m| SecureOrTune::Secure(m));
            let tune_op = c.tune().map(|m| SecureOrTune::Tune(m));
            secure_op.or(tune_op)
        })
        .ok_or(Error::from(ErrorKind::FailToHandshake))
}

fn is_open_ok(frame: &Frame) -> Result<&OpenOkMethod, Error> {
    frame
        .method()
        .and_then(|m| m.connection())
        .and_then(|c| c.open_ok())
        .ok_or(Error::from(ErrorKind::FailToHandshake))
}
