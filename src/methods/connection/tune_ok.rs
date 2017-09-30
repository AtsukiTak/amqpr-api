use amqpr_codec::methods::args::*;
use amqpr_codec::frame::method::MethodPayload;

pub fn tune_ok(tune: &MethodPayload) -> MethodPayload {
    let args = &tune.arguments;

    let channel_max = Argument::Short(channel_max(args));
    let frame_max = Argument::Long(frame_max(args));
    let heartbeat = Argument::Short(heartbeat(args));

    let payload = MethodPayload {
        class_id: 10,
        method_id: 31,
        arguments: Arguments(vec![channel_max, frame_max, heartbeat]),
    };

    payload
}


fn channel_max(args: &Arguments) -> u16 {
    match &args[0] {
        &Argument::Short(max) => max,
        a => panic!("Invalid argument. We expect Short but found {:?}", a),
    }
}


fn frame_max(args: &Arguments) -> u32 {
    match &args[1] {
        &Argument::Long(max) => max,
        a => panic!("Invalid argument. We expect Long but found {:?}", a),
    }
}


fn heartbeat(args: &Arguments) -> u16 {
    match &args[2] {
        &Argument::Short(heartbeat) => heartbeat,
        a => panic!("Invalid argument. We expect Short but found {:?}", a),
    }
}
