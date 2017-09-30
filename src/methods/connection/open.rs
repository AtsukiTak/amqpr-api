use amqpr_codec::methods::args::*;
use amqpr_codec::frame::method::MethodPayload;

pub fn open(vhost: &str) -> MethodPayload {
    let virtual_host = Argument::ShortString(virtual_host(vhost));
    let reserved_1 = Argument::ShortString(ShortString("".into()));
    let reserved_2 = Argument::Bits(0b00000001);

    let payload = MethodPayload {
        class_id: 10,
        method_id: 40,
        arguments: Arguments(vec![virtual_host, reserved_1, reserved_2]),
    };

    payload
}


fn virtual_host(vhost: &str) -> ShortString {
    ShortString(vhost.into())
}
