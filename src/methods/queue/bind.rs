use amqpr_codec::frame::method::*;
use amqpr_codec::methods::args::*;


pub struct BindArguments {
    pub queue_name: String,
    pub exchange_name: String,
    pub routing_key: String,
    pub no_wait: bool,
}

impl Default for BindArguments {
    fn default() -> Self {
        BindArguments {
            queue_name: "".into(),
            exchange_name: "".into(),
            routing_key: "".into(),
            no_wait: false,
        }
    }
}


pub fn bind(args: BindArguments) -> MethodPayload {
    let reserved_1 = Argument::Short(0);
    let queue_name = Argument::ShortString(ShortString(args.queue_name.clone()));
    let exchange_name = Argument::ShortString(ShortString(args.exchange_name.clone()));
    let routing_key = Argument::ShortString(ShortString(args.routing_key.clone()));
    let bits = Argument::Bits(bits_from_some_arg(&args));
    let table = Argument::FieldTable(FieldTable(Vec::new()));

    let arguments =
        Arguments(vec![reserved_1, queue_name, exchange_name, routing_key, bits, table]);

    MethodPayload {
        class_id: 50,
        method_id: 20,
        arguments: arguments,
    }
}


fn bits_from_some_arg(args: &BindArguments) -> u8 {
    (args.no_wait as u8 * 0b_0000_0001_u8)
}
