extern crate amqpr_api;
extern crate amqpr_codec;
extern crate tokio_core;
extern crate futures;
extern crate bytes;
extern crate log;
extern crate log4rs;

use tokio_core::reactor::{Core, Interval};
use tokio_core::net::TcpStream;
use futures::{Future, Stream, Sink};

use bytes::Bytes;

use amqpr_codec::Frame;

use amqpr_api::{start_handshake, declare_exchange_wait, open_channel, publish, AmqpSocket};
use amqpr_api::exchange::declare::{ExchangeType, DeclareExchangeOption};
use amqpr_api::basic::publish::{PublishOption, Published};
use amqpr_api::handshake::SimpleHandshaker;
use amqpr_api::errors::*;


const LOCAL_CHANNEL_ID: u16 = 42;

fn main() {
    logger();

    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let handshaker = SimpleHandshaker {
        user: "guest".into(),
        pass: "guest".into(),
        virtual_host: "/".into(),
    };

    let future = TcpStream::connect(&"127.0.0.1:5672".parse().unwrap(), &core.handle())
        .map_err(|e| Error::from(e))
        .and_then(|socket| start_handshake(handshaker, socket))
        .and_then(|socket| Ok(socket.split()))
        .and_then(|(outcome, income)| Ok((income.map_err(|e| Error::from(e)), outcome.sink_map_err(|e| Error::from(e)))))
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
        .and_then(move |(_income, outcome)| {
            let interval = Interval::new(std::time::Duration::new(1, 0), &handle).unwrap();
            interval
                .map_err(|e| Error::from(e))
                .fold(outcome, |outcome, _| {
                    inner_publish(outcome)
                })
        });

    core.run(future).unwrap();
}

fn inner_publish<Out>(outcome: Out) -> Published<Out>
where
    Out: Sink<SinkItem = Frame>,
{
    let option = PublishOption {
        exchange: "example".into(),
        routing_key: "".into(),
        is_mandatory: false,
        is_immediate: false,
    };
    let bytes = Bytes::from_static(b"pubish example");
    publish(outcome, LOCAL_CHANNEL_ID, bytes, option)
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
