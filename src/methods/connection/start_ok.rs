use amqpr_codec::methods::args::*;
use amqpr_codec::frame::method::MethodPayload;

pub fn start_ok(start: &MethodPayload, user: &str, pass: &str) -> MethodPayload {
    let args = &start.arguments;

    let properties = Argument::FieldTable(client_properties());
    let mechanism = Argument::ShortString(select_security_mechanism(args, "PLAIN"));
    let security_data = Argument::LongString(plain_security_response_data("", user, pass));
    let locale = Argument::ShortString(select_locale(args, "en_US"));

    let payload = MethodPayload {
        class_id: 10,
        method_id: 11,
        arguments: Arguments(vec![properties, mechanism, security_data, locale]),
    };

    payload
}


// TODO
/// Fow now, there is only fake property.
/// We shoud improve this.
fn client_properties() -> FieldTable {
    let product = FieldArgument::LongString(LongString(format!("amqpr")));
    let version = FieldArgument::LongString(LongString(format!("0.0.1")));
    let platform = FieldArgument::LongString(LongString(format!("Rust stable 1.19")));
    let copyright = FieldArgument::LongString(LongString(format!("Copyright (C) 2017 Atsuki-Tak")));
    let information = FieldArgument::LongString(LongString(
        format!("Do not use this library for production"),
    ));
    FieldTable(vec![
        (format!("product"), product),
        (format!("version"), version),
        (format!("platform"), platform),
        (format!("copyright"), copyright),
        (format!("information"), information),
    ])
}


// TODO
/// For now, we just select "PLAIN" mode.
/// If server does not support "PLAIN", it panics.
fn select_security_mechanism(args: &Arguments, mechanism: &str) -> ShortString {
    match &args[3] {
        &Argument::LongString(LongString(ref s)) => {
            if s.contains(mechanism) {
                ShortString(mechanism.into())
            } else {
                panic!("[ {} ] does not contain [ {} ]", s, mechanism)
            }
        }
        a => panic!("Invalid argument. We expect LongString but found {:?}", a),
    }
}



// TODO
// https://www.rfc-editor.org/rfc/pdfrfc/rfc4616.txt.pdf
fn plain_security_response_data(authzid: &str, authcid: &str, password: &str) -> LongString {
    LongString(format!("{}\0{}\0{}", authzid, authcid, password))
}


// TODO
/// Fow now, we just return "en_US"
fn select_locale(args: &Arguments, locale: &str) -> ShortString {
    match &args[4] {
        &Argument::LongString(LongString(ref s)) => {
            if s.contains(locale) {
                ShortString(locale.into())
            } else {
                panic!("[ {} ] does not contain [ {} ]", s, locale)
            }
        }
        a => panic!("Invalid argument. We expect LongString but found {:?}", a),
    }
}
