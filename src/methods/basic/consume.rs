use amqpr_codec::frame::method::*;
use amqpr_codec::methods::args::*;


pub struct ConsumeArguments {
    pub queue_name: String,
    pub consumer_tag: String,
    pub no_local: bool,
    pub no_ack: bool,
    pub exclusive: bool,
    pub no_wait: bool,
}

impl Default for ConsumeArguments {
    fn default() -> Self {
        ConsumeArguments {
            queue_name: "".into(),
            consumer_tag: "".into(),
            no_local: false,
            no_ack: false,
            exclusive: false,
            no_wait: false,
        }
    }
}


pub fn consume(args: ConsumeArguments) -> MethodPayload {
    let reserved_1 = Argument::Short(0);
    let bits = Argument::Bits(bits_from_some_arg(&args));
    let queue_name = Argument::ShortString(ShortString(args.queue_name));
    let consumer_tag = Argument::ShortString(ShortString(args.consumer_tag));
    let table = Argument::FieldTable(FieldTable(Vec::new()));

    let arguments = Arguments(vec![reserved_1, queue_name, consumer_tag, bits, table]);

    MethodPayload {
        class_id: 60,
        method_id: 20,
        arguments: arguments,
    }
}


fn bits_from_some_arg(args: &ConsumeArguments) -> u8 {
    (args.no_local as u8 * 0b_0000_0001_u8) + (args.no_ack as u8 * 0b_0000_0010_u8) +
        (args.exclusive as u8 * 0b_0000_0100_u8) + (args.no_wait as u8 * 0b_0000_1000_u8)
}
