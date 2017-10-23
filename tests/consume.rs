extern crate amqpr_api;
extern crate tokio_core;
extern crate futures;
extern crate bytes;
extern crate log;
extern crate log4rs;

use tokio_core::reactor::Core;
use tokio_core::net::TcpStream;
use futures::Future;

use bytes::Bytes;

use amqpr_api::{start_handshake, declare_exchange, open_channel, publish, bind_queue,
                declare_queue, start_consume, receive_delivered};
use amqpr_api::exchange::declare::{ExchangeType, DeclareExchangeOption};
use amqpr_api::queue::{DeclareQueueOption, BindQueueOption};
use amqpr_api::basic::{PublishOption, StartConsumeOption};
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
        .and_then(|socket| open_channel(socket, LOCAL_CHANNEL_ID))
        .and_then(|socket| {
            let option = DeclareExchangeOption {
                name: "consume_test".into(),
                typ: ExchangeType::Fanout,
                is_passive: false,
                is_durable: false,
                is_auto_delete: true,
                is_internal: false,
                is_no_wait: false,
            };
            declare_exchange(socket, LOCAL_CHANNEL_ID, option)
        })
        .and_then(|socket| {
            let option = DeclareQueueOption {
                name: "consume_test".into(),
                is_passive: false,
                is_durable: false,
                is_exclusive: false,
                is_auto_delete: true,
                is_no_wait: false,
            };
            declare_queue(socket, LOCAL_CHANNEL_ID, option).map(|(_result, socket)| socket)
        })
        .and_then(|socket| {
            let option = BindQueueOption {
                queue: "consume_test".into(),
                exchange: "consume_test".into(),
                routing_key: "".into(),
                is_no_wait: false,
            };
            bind_queue(socket, LOCAL_CHANNEL_ID, option)
        })
        .and_then(|socket| {
            let option = PublishOption {
                exchange: "consume_test".into(),
                routing_key: "".into(),
                is_mandatory: false,
                is_immediate: false,
            };
            let bytes = Bytes::from_static(b"pubish test");
            publish(socket, LOCAL_CHANNEL_ID, bytes, option)
        })
        .and_then(|socket| {
            let option = StartConsumeOption {
                queue: "consume_test".into(),
                consumer_tag: "".into(),
                is_no_local: false,
                is_no_ack: true,
                is_exclusive: false,
                is_no_wait: false,
            };
            start_consume(socket, LOCAL_CHANNEL_ID, option)
        })
        .and_then(|socket| receive_delivered(socket));

    core.run(future).unwrap();
}


fn logger() {
    use log::LogLevelFilter;
    use log4rs::append::console::ConsoleAppender;
    use log4rs::config::{Appender, Config, Root};
    let stdout = ConsoleAppender::builder().build();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(
            LogLevelFilter::Info,
        ))
        .unwrap();

    log4rs::init_config(config).unwrap();
}
