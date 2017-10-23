error_chain! {
    types {
        Error, ErrorKind, ResultExt;
    }

    foreign_links {
        Io(::std::io::Error);
    }

    errors {
        UnexpectedConnectionClose {
            description("Connection was closed unexpectedly")
            display("Connection was closed unexpectedly")
        }
        FailToHandshake {
            description("Fail to complete handshake")
            display("Fail to complete handshake")
        }
        UnexpectedFrame {
            description("Receive unexpected frame")
            display("Receive unexpected frame")
        }
    }
}
