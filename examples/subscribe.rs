extern crate amqpr_api;
extern crate bytes;
extern crate futures;
extern crate log4rs;
extern crate log;
extern crate tokio_core;

use tokio_core::reactor::Core;
use tokio_core::net::TcpStream;
use futures::{Future, Stream};

use amqpr_api::{bind_queue, declare_exchange, declare_queue, open_channel, start_handshake,
                subscribe_stream};
use amqpr_api::exchange::declare::{DeclareExchangeOption, ExchangeType};
use amqpr_api::queue::{BindQueueOption, DeclareQueueOption};
use amqpr_api::basic::StartConsumeOption;
use amqpr_api::handshake::SimpleHandshaker;
use amqpr_api::errors::*;

const LOCAL_CHANNEL_ID: u16 = 42;
const EXCHANGE_NAME: &'static str = "example";
const QUEUE_NAME: &'static str = "example";

fn main() {
    let (addr, user, pass) = get_args();

    logger();

    let mut core = Core::new().unwrap();

    let handshaker = SimpleHandshaker {
        user: user,
        pass: pass,
        virtual_host: "/".into(),
    };

    let future = TcpStream::connect(&addr.parse().unwrap(), &core.handle())
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


fn get_args() -> (String, String, String) {
    use clap::{App, Arg};

    let matches = App::new("amqpr-api broadcast example")
        .arg(
            Arg::with_name("amqp_addr")
                .short("a")
                .long("amqp_addr")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("user")
                .short("u")
                .long("user")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("pass")
                .short("p")
                .long("pass")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    (
        matches.value_of("amqp_addr").unwrap().into(),
        matches.value_of("user").unwrap().into(),
        matches.value_of("pass").unwrap().into(),
    )
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
