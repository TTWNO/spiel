use hound::{SampleFormat, WavSpec, WavWriter};
use itertools::Itertools;
use spiel::{read_message_type, MessageType};

fn main() {
	let mut data: &[u8] = include_bytes!("../test.wav");
	let mut header = false;
	let spec = WavSpec {
		channels: 1,
		sample_rate: 22050,
		bits_per_sample: 16,
		sample_format: SampleFormat::Int,
	};
	let mut writer = WavWriter::create("out.wav", spec).expect("Can make wave writer!");
	for _ in 0..55 {
		let (data_next, msg) =
			read_message_type(data, header).expect("to be able to read data");
		header = true;
		if let MessageType::Audio { samples_offset, samples_len } = msg {
			let ch = &data[samples_offset..samples_len];
			for (l, h) in ch.iter().tuples() {
				let sample = i16::from_le_bytes([*l, *h]);
				writer.write_sample(sample).expect("Can write to file");
			}
		}
		data = data_next;
	}
}
