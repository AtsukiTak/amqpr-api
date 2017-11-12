extern crate amqpr_api;
extern crate tokio_core;
extern crate futures;
extern crate bytes;
extern crate log;
extern crate log4rs;

use tokio_core::reactor::Core;
use tokio_core::net::TcpStream;
use futures::{Future, Stream, Sink};
use futures::stream::unfold;

use amqpr_api::{start_handshake, declare_exchange_wait, open_channel, bind_queue, declare_queue,
                start_consume, receive_delivered};
use amqpr_api::exchange::declare::{ExchangeType, DeclareExchangeOption};
use amqpr_api::queue::{DeclareQueueOption, BindQueueOption};
use amqpr_api::basic::StartConsumeOption;
use amqpr_api::handshake::SimpleHandshaker;
use amqpr_api::errors::*;


const LOCAL_CHANNEL_ID: u16 = 42;

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
        .and_then(|socket| Ok(socket.split()))
        .and_then(|(outcome, income)| {
            Ok((
                income.map_err(|e| Error::from(e)),
                outcome.sink_map_err(|e| Error::from(e)),
            ))
        })
        .and_then(|(income, outcome)| {
            open_channel(income, outcome, LOCAL_CHANNEL_ID)
        })
        .and_then(|(income, outcome)| {
            let option = DeclareExchangeOption {
                name: "example".into(),
                typ: ExchangeType::Fanout,
                is_passive: false,
                is_durable: false,
                is_auto_delete: true,
                is_internal: false,
                is_no_wait: true,
            };
            declare_exchange_wait(income, outcome, LOCAL_CHANNEL_ID, option)
        })
        .and_then(|(income, outcome)| {
            let option = DeclareQueueOption {
                name: "example".into(),
                is_passive: false,
                is_durable: false,
                is_exclusive: false,
                is_auto_delete: true,
                is_no_wait: false,
            };
            declare_queue(income, outcome, LOCAL_CHANNEL_ID, option)
        })
        .and_then(|(_res, income, outcome)| {
            let option = BindQueueOption {
                queue: "example".into(),
                exchange: "example".into(),
                routing_key: "".into(),
                is_no_wait: false,
            };
            bind_queue(income, outcome, LOCAL_CHANNEL_ID, option)
        })
        .and_then(|(income, outcome)| {
            let option = StartConsumeOption {
                queue: "example".into(),
                consumer_tag: "".into(),
                is_no_local: false,
                is_no_ack: true,
                is_exclusive: false,
                is_no_wait: false,
            };
            start_consume::<_, _, Error>(income, outcome, LOCAL_CHANNEL_ID, option)
        })
        .and_then(|(income, _outcome)| {
            unfold(income, |income| Some(receive_delivered(income)))
                .for_each(|bytes| Ok(println!("{:?}", bytes)))
        });

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
