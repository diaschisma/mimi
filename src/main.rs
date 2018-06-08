extern crate portmidi;
extern crate rosc;
extern crate miosc;

extern crate clap;

fn midi_pitch(midi: u8, edo: f32, ref_key: f32, ref_pitch: f32) -> f32 {
    (midi as f32 - ref_key) * 12.0 / edo + ref_pitch
}

fn encode_miosc(msg: miosc::MioscMessage) -> Vec<u8> {
    let packet = rosc::OscPacket::Message(msg.into());

    rosc::encoder::encode(&packet).unwrap()
}

fn main() {
    let matches =
        clap::App::new("Mimi")
        .version(env!("CARGO_PKG_VERSION"))
        .about("A linear MIDI to Miosc converter")
        .arg(
            clap::Arg::with_name("ref-key")
            .short("k")
            .long("ref-key")
            .value_name("key")
            .help("The MIDI note of the reference pitch")
            .takes_value(true)
        ).arg(
            clap::Arg::with_name("ref-pitch")
            .short("p")
            .long("ref-pitch")
            .value_name("semitones")
            .help("The reference pitch, in 12edo steps from MIDI 0")
            .takes_value(true)
        ).arg(
            clap::Arg::with_name("edo")
            .short("e")
            .long("edo")
            .value_name("edo")
            .help("The number of divisions of the octave")
            .takes_value(true)
        ).arg(
            clap::Arg::with_name("address")
            .help("The OSC address to send messages to")
            .index(1)
        ).get_matches();

    let midi = portmidi::PortMidi::new().unwrap();
    let mut midi_in = midi.default_input_port(1024).unwrap();

    let address = matches.value_of("address").unwrap_or("localhost:3579");
    let socket = ::std::net::UdpSocket::bind("localhost:9753").unwrap();

    let ref_key = matches.value_of("ref_key").and_then(|k| k.parse().ok()).unwrap_or(60.0);
    let ref_pitch = matches.value_of("ref_pitch").and_then(|k| k.parse().ok()).unwrap_or(60.0);
    let edo = matches.value_of("edo").and_then(|k| k.parse().ok()).unwrap_or(31.0);

    loop {
        if let Ok(Some(ev)) = midi_in.read() {
            use miosc::MioscMessage;

            let msg = ev.message;

            match msg.status {
                0x90 => {
                    let msg = MioscMessage::NoteOn(
                        msg.data1 as _,
                        midi_pitch(msg.data1, edo, ref_key, ref_pitch),
                        msg.data2 as f32 / 127.0
                    );

                    let bytes = encode_miosc(msg);
                    drop(socket.send_to(&bytes, address))
                },
                0x80 => {
                    let msg = MioscMessage::NoteOff(
                        msg.data1 as _,
                    );

                    let bytes = encode_miosc(msg);
                    drop(socket.send_to(&bytes, address))
                },
                _ => (),
            }
        }

        let dt = ::std::time::Duration::from_millis(8);
        ::std::thread::sleep(dt)
    }
}
