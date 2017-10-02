use amqpr_codec::frame::method::*;
use amqpr_codec::methods::args::*;


#[derive(Debug, Clone)]
pub struct DeclareArguments {
    pub exchange_name: String,
    pub exchange_type: String,
    pub passive: bool,
    pub durable: bool,
    pub auto_delete: bool,
    pub internal: bool,
    pub no_wait: bool,
}

impl Default for DeclareArguments {
    fn default() -> Self {
        DeclareArguments {
            exchange_name: "".into(),
            exchange_type: "fanout".into(),
            passive: false,
            durable: false,
            auto_delete: false,
            internal: false,
            no_wait: false,
        }
    }
}


pub fn declare(args: DeclareArguments) -> MethodPayload {
    let reserved_1 = Argument::Short(0);
    let exchange_name_arg = Argument::ShortString(ShortString(args.exchange_name));
    let exchange_type_arg = Argument::ShortString(ShortString(args.exchange_type));
    let bits = Argument::Bits(bits_from_some_arg(
        args.passive,
        args.durable,
        args.auto_delete,
        args.internal,
        args.no_wait,
    ));
    let table = Argument::FieldTable(FieldTable(Vec::new()));

    let arguments = Arguments(vec![
        reserved_1,
        exchange_name_arg,
        exchange_type_arg,
        bits,
        table,
    ]);

    MethodPayload {
        class_id: 40,
        method_id: 10,
        arguments: arguments,
    }
}


fn bits_from_some_arg(
    passive: bool,
    durable: bool,
    auto_delete: bool,
    internal: bool,
    no_wait: bool,
) -> u8 {
    (passive as u8 * 0b00000001_u8) + (durable as u8 * 0b00000010_u8) +
        (auto_delete as u8 * 0b00000100_u8) + (internal as u8 * 0b00001000_u8) +
        (no_wait as u8 * 0b00010000_u8)
}
