use amqpr_codec::frame::method::*;
use amqpr_codec::methods::args::*;


pub struct DeclareArguments {
    pub queue_name: &'static str,
    pub passive: bool,
    pub durable: bool,
    pub exclusive: bool,
    pub auto_delete: bool,
    pub no_wait: bool,
}

impl Default for DeclareArguments {
    fn default() -> Self {
        DeclareArguments {
            queue_name: "",
            passive: false,
            durable: false,
            exclusive: false,
            auto_delete: false,
            no_wait: false,
        }
    }
}


pub fn declare(args: DeclareArguments) -> MethodPayload {
    let reserved_1 = Argument::Short(0);
    let queue_name = Argument::ShortString(ShortString(args.queue_name.into()));
    let bits = Argument::Bits(bits_from_some_arg(&args));
    let table = Argument::FieldTable(FieldTable(Vec::new()));

    let arguments = Arguments(vec![reserved_1, queue_name, bits, table]);

    MethodPayload {
        class_id: 50,
        method_id: 10,
        arguments: arguments,
    }
}


fn bits_from_some_arg(args: &DeclareArguments) -> u8 {
    (args.passive as u8 * 0b00000001_u8) + (args.durable as u8 * 0b00000010_u8) +
        (args.exclusive as u8 * 0b00000100_u8) + (args.auto_delete as u8 * 0b00001000_u8) +
        (args.no_wait as u8 * 0b00010000_u8)
}
