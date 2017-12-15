amqpr-api
===

A tokio future based amqp api library.


![Apache-2.0 licensed](https://img.shields.io/badge/License-Apache%202.0-blue.svg)
[![Crates.io](https://img.shields.io/crates/v/amqpr-api.svg)](https://crates.io/crates/amqpr-api)


This library provides low lovel api. I recommend you to use [`amqpr`](https://github.com/AtsukiTak/amqpr) library instead.


# Examples

## Establish connection
```rust
use amqpr_api::handshake::{start_handshake, SimpleHandshaker};
use tokio_core::reactor::Core;
use tokio_core::net::TcpStream;

let mut core = Core::new().unwrap();

let handshaker = SimpleHandshaker {
  user: "guest".into(),
  pass: "guest".into(),
  virtual_host: "/".into(),
};

let future = TcpStream::connect(&"127.0.0.1:5672".parse().unwrap(), &core.handle())
    .map_err(|e| Error::from(e))
    .and_then(move |socket| start_handshake(handshaker, socket));

let socket = core.run(future).unwrap();
```
