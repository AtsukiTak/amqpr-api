extern crate amqpr_api;
extern crate bytes;
extern crate futures;
extern crate log4rs;
extern crate log;
extern crate tokio_core;

use tokio_core::reactor::Core;
use tokio_core::net::TcpStream;
use futures::Future;

use amqpr_api::start_handshake;
use amqpr_api::handshake::SimpleHandshaker;
use amqpr_api::errors::*;

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
        .and_then(|socket| start_handshake(handshaker, socket));

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

    let _handle = log4rs::init_config(config).unwrap();
}
