use spiel::{
    Chunk,
    ChunkHold,
    read_chunk,
};
use hound::{
    WavWriter,
    SampleFormat,
    WavSpec,
};
use itertools::Itertools;

fn main() {
    let mut data: &[u8] = include_bytes!("../test.wav");
    let mut chunk = Chunk::Version("no");
    let mut header = false;
    let spec = WavSpec {
        channels: 1,
        sample_rate: 22050,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    let mut writer = WavWriter::create("out.wav", spec).expect("Can make wave writer!");
    for _ in 0..55 {
        (data, chunk) = read_chunk(data, header).expect("to be able to read data");
        header = true;
        if let Chunk::Audio(ch) = chunk {
            println!("DATA: {:?}", ch);
            for (l,h) in ch.buf.iter().tuples() {
                let sample = i16::from_le_bytes([*l,*h]);
                writer.write_sample(sample).expect("Can write to file");
            }
        }
    }
}
