//! amqpr-api is AMQP client api library.
//! You can talk with AMQP server via channel controller provided by this crate.
//! There is two kind of channel controllers; GlobalChannelController and LocalChannelController.
//!
//!                             +-------------+
//!                             | AMQP Server |
//!                             +------+------+
//!                                    ^
//!                                    |
//!                                    | TCP
//!                                    |
//!                            +-------+-------+
//!                        +-> | SocketHandler | <-+
//!                        |   +---------------+   |
//!                        |                       |
//!       SocketController |                       | SocketController
//!                 +------+-------+        +------+--------+
//!                 | LocalChannel |        | GlobalChannel |
//!                 +------+-------+        +------+--------+
//!                        ^                       ^
//! +----------------------+-----------------------+--------------------------+
//!                        |                       |           Library Interface
//!                        |                       |
//!                        |                       |
//!       +----------------+-------+        +------+------------------+
//!       | LocalChannelController |        | GlobalChannelController |
//!       +------------------------+        +-------------------------+
//!

extern crate tokio_core;
extern crate tokio_io;
extern crate futures;
extern crate bytes;

#[macro_use]
extern crate log;

extern crate amqpr_codec;




mod socket;
mod channel;
pub mod methods;

pub use socket::open as socket_open;
pub use channel::{GlobalChannelController, LocalChannelController};
