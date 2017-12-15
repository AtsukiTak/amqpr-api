extern crate amqpr_api;
extern crate tokio_core;
extern crate futures;
extern crate bytes;
extern crate log;
extern crate log4rs;

use tokio_core::reactor::Core;
use tokio_core::net::TcpStream;
use futures::{Future, Stream};

use amqpr_api::{start_handshake, declare_exchange, open_channel, bind_queue, declare_queue,
                subscribe_stream};
use amqpr_api::exchange::declare::{ExchangeType, DeclareExchangeOption};
use amqpr_api::queue::{DeclareQueueOption, BindQueueOption};
use amqpr_api::basic::StartConsumeOption;
use amqpr_api::handshake::SimpleHandshaker;
use amqpr_api::errors::*;


const LOCAL_CHANNEL_ID: u16 = 42;
const EXCHANGE_NAME: &'static str = "example";
const QUEUE_NAME: &'static str = "example";

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
                name: EXCHANGE_NAME.into(),
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
                name: QUEUE_NAME.into(),
                is_passive: false,
                is_durable: false,
                is_exclusive: false,
                is_auto_delete: true,
            };
            declare_queue(LOCAL_CHANNEL_ID, socket, option)
        })
        .and_then(|(res, socket)| {
            let option = BindQueueOption {
                queue: res.queue,
                exchange: EXCHANGE_NAME.into(),
                routing_key: "".into(),
            };
            bind_queue(LOCAL_CHANNEL_ID, socket, option)
        })
        .map(|socket| {
            let option = StartConsumeOption {
                queue: QUEUE_NAME.into(),
                consumer_tag: "".into(),
                is_no_local: false,
                is_no_ack: true,
                is_exclusive: true,
            };
            subscribe_stream(LOCAL_CHANNEL_ID, socket, option)
        });

    let stream = core.run(future).unwrap();

    let _ = core.run(stream.for_each(|item| Ok(println!("{:?}", item))));
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
