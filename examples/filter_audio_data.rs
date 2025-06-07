use hound::{SampleFormat, WavSpec, WavWriter};
use itertools::Itertools;
use spiel::{read_message, Message};

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
		let (offset, msg) = read_message(data, header).expect("to be able to read data");
		header = true;
		if let Message::Audio(samples) = msg {
			for (l, h) in samples.iter().tuples() {
				let sample = i16::from_le_bytes([*l, *h]);
				writer.write_sample(sample).expect("Can write to file");
			}
		}
		data = &data[offset..];
	}
}
