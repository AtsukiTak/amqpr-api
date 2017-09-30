use amqpr_codec::frame::method::*;
use amqpr_codec::methods::args::*;


#[derive(Debug, Clone)]
pub struct PublishArguments {
    pub exchange_name: String,
    pub routing_key: String,
    pub mandatory: bool,
    pub immediate: bool,
}

impl Default for PublishArguments {
    fn default() -> Self {
        PublishArguments {
            exchange_name: "".into(),
            routing_key: "".into(),
            mandatory: false,
            immediate: false,
        }
    }
}


pub fn publish(args: PublishArguments) -> MethodPayload {
    let reserved_1 = Argument::Short(0);
    let bits = Argument::Bits(bits_from_some_arg(&args));
    let exchange_name = Argument::ShortString(ShortString(args.exchange_name));
    let routing_key = Argument::ShortString(ShortString(args.routing_key));

    let arguments = Arguments(vec![reserved_1, exchange_name, routing_key, bits]);

    MethodPayload {
        class_id: 60,
        method_id: 40,
        arguments: arguments,
    }
}


fn bits_from_some_arg(args: &PublishArguments) -> u8 {
    (args.mandatory as u8 * 0b_0000_0001_u8) + (args.immediate as u8 * 0b_0000_0010_u8)
}
