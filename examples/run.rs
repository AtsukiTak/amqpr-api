extern crate amqpr_api;
extern crate tokio_core;
extern crate futures;
extern crate bytes;
extern crate log4rs;

use tokio_core::reactor::Core;
use futures::{Future, Stream};

use bytes::Bytes;

use amqpr_api::socket_open;
use amqpr_api::methods::{exchange, queue, basic};

fn main() {
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();

    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let controller_future = socket_open(&"127.0.0.1:5672".parse().unwrap(), &handle, "test".into(), "test".into())

        // Open new channel to perform some operation
        .and_then(|controller| controller.declare_local_channel(42));


    let (_global_con, local_con) = core.run(controller_future).unwrap();

    // Declare Exchange
    let args = exchange::DeclareArguments {
        exchange_name: "test".into(),
        exchange_type: "fanout".into(),
        durable: true,
        auto_delete: true,
        ..Default::default()
    };
    local_con.declare_exchange(args);

    // Declare Queue
    let args = queue::DeclareArguments {
        // If you leave this field empty, Rabbitmq server will chose queue_name randomly.
        queue_name: "",
        durable: true,
        auto_delete: true,
        ..Default::default()
    };
    let queue_name_future = local_con.declare_queue(args);
    let queue_name = core.run(queue_name_future).unwrap();

    // Bind declared Queue to Exchange
    let args = queue::BindArguments {
        queue_name: queue_name.clone(),
        exchange_name: "test".into(),
        ..Default::default()
    };
    local_con.bind_queue(args);

    // Publish contents
    let args = basic::PublishArguments {
        exchange_name: "test".into(),
        routing_key: "".into(),
        ..Default::default()
    };
    let bytes = Bytes::from_static(b"hello world");
    local_con.publish(args, bytes);


    // Consume contents from test-comsuming queue.
    let args = basic::ConsumeArguments {
        queue_name: queue_name,
        no_ack: true,
        ..Default::default()
    };
    let data_stream = local_con.consume(args);

    let future = data_stream.for_each(|byte| {
        println!("You got a new data !!!\n\n\n\n\n{:?}", byte);
        Ok(())
    });
    core.run(future).unwrap();


}
