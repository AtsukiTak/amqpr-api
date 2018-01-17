extern crate amqpr_api;
extern crate amqpr_codec;
extern crate bytes;
extern crate futures;
extern crate log4rs;
extern crate log;
extern crate tokio_core;

use tokio_core::reactor::Core;
use tokio_core::net::TcpStream;
use futures::Future;

use bytes::Bytes;

use amqpr_codec::args::Properties;
use amqpr_api::{bind_queue, declare_exchange, declare_queue, get_delivered, open_channel, publish,
                start_consume, start_handshake};
use amqpr_api::exchange::declare::{DeclareExchangeOption, ExchangeType};
use amqpr_api::queue::{BindQueueOption, DeclareQueueOption};
use amqpr_api::basic::{PublishItem, PublishOption, StartConsumeOption};
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
                name: "consume_test".into(),
                typ: ExchangeType::Fanout,
                is_passive: false,
                is_durable: false,
                is_auto_delete: true,
                is_internal: false,
            };
            declare_exchange(LOCAL_CHANNEL_ID, socket, option)
        })
        .and_then(|socket| {
            let option = DeclareQueueOption {
                name: "consume_test".into(),
                is_passive: false,
                is_durable: false,
                is_exclusive: false,
                is_auto_delete: true,
            };
            declare_queue(LOCAL_CHANNEL_ID, socket, option).map(|(_result, socket)| socket)
        })
        .and_then(|socket| {
            let option = BindQueueOption {
                queue: "consume_test".into(),
                exchange: "consume_test".into(),
                routing_key: "".into(),
            };
            bind_queue(LOCAL_CHANNEL_ID, socket, option)
        })
        .and_then(|socket| {
            let option = PublishOption {
                exchange: "consume_test".into(),
                routing_key: "".into(),
                is_mandatory: false,
                is_immediate: false,
            };
            let bytes = Bytes::from_static(b"pubish test");
            let item = PublishItem {
                meta: option,
                header: Properties::new(),
                body: bytes,
            };
            publish(LOCAL_CHANNEL_ID, socket, item)
        })
        .and_then(|socket| {
            let option = StartConsumeOption {
                queue: "consume_test".into(),
                consumer_tag: "".into(),
                is_no_local: false,
                is_no_ack: true,
                is_exclusive: false,
            };
            start_consume(LOCAL_CHANNEL_ID, socket, option)
        })
        .and_then(|socket| get_delivered(socket));

    core.run(future).unwrap();
}

fn logger() {
    use log::LevelFilter;
    use log4rs::append::console::ConsoleAppender;
    use log4rs::config::{Appender, Config, Root};
    let stdout = ConsoleAppender::builder().build();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(LevelFilter::Info))
        .unwrap();

    log4rs::init_config(config).unwrap();
}
