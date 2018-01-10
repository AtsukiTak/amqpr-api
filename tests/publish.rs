extern crate amqpr_api;
extern crate amqpr_codec;
extern crate tokio_core;
extern crate futures;
extern crate bytes;
extern crate log;
extern crate log4rs;

use tokio_core::reactor::Core;
use tokio_core::net::TcpStream;
use futures::Future;

use bytes::Bytes;

use amqpr_api::{start_handshake, declare_exchange, open_channel, publish};
use amqpr_api::exchange::declare::{ExchangeType, DeclareExchangeOption};
use amqpr_api::basic::publish::PublishOption;
use amqpr_api::handshake::SimpleHandshaker;
use amqpr_api::errors::*;


const LOCAL_CHANNEL_ID: u16 = 42;

#[test]
fn main() {
    logger();

    let mut core = Core::new().unwrap();

    let handshaker = SimpleHandshaker {
        user: "guest".into(),
        pass: "guest".into(),
        virtual_host: "/".into(),
    };

    let future = TcpStream::connect(&"127.0.0.1:5672".parse().unwrap(), &core.handle())
        .map_err(|e| Error::from(e))
        .and_then(|socket| start_handshake(handshaker, socket))
        .and_then(|socket| open_channel(LOCAL_CHANNEL_ID, socket))
        .and_then(|socket| {
            let option = DeclareExchangeOption {
                name: "publish_test".into(),
                typ: ExchangeType::Fanout,
                is_passive: false,
                is_durable: false,
                is_auto_delete: true,
                is_internal: false,
            };
            declare_exchange(LOCAL_CHANNEL_ID, socket, option)
        })
        .and_then(move |socket| {
            let option = PublishOption {
                exchange: "publish_test".into(),
                routing_key: "".into(),
                is_mandatory: false,
                is_immediate: false,
            };
            let bytes = Bytes::from_static(b"pubish test");
            publish(LOCAL_CHANNEL_ID, socket, bytes, option)
        });

    core.run(future).unwrap();
}


fn logger() {
    use log::LevelFilter;
    use log4rs::append::console::ConsoleAppender;
    use log4rs::config::{Appender, Config, Root};
    let stdout = ConsoleAppender::builder().build();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(
            LevelFilter::Info,
        ))
        .unwrap();

    log4rs::init_config(config).unwrap();
}
