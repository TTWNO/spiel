use std::{
	io::{PipeWriter, Write},
	os::fd::OwnedFd,
	time::Duration,
};

use spiel::{write_message, Event, EventType, Message, Voice, VoiceFeatureSet};
use tokio::time::sleep;
use zbus::{connection::Builder, fdo::Error, interface, zvariant::Fd};

struct MySpeechProvider {
	voices: Vec<Voice>,
}

#[interface(name = "org.freedesktop.Speech.Provider")]
impl MySpeechProvider {
	#[zbus(property)]
	async fn voices(&self) -> Vec<Voice> {
		self.voices.clone()
	}
	#[zbus(property)]
	async fn name(&self) -> String {
		"Silly Provider!".to_string()
	}
	#[allow(clippy::too_many_arguments)]
	async fn synthesize(
		&self,
		pipe_fd: Fd<'_>,
		_text: &str,
		_voice_id: &str,
		_pitch: f64,
		_rate: f64,
		_is_ssml: bool,
		_language: &str,
	) {
		println!("Received a syntheiszer event!");
		// find a voice that matches the language &str
		// actually synthesize text,
		// etc.
		//
		// We are just gonna write a simple `Message::Event`
		let header = Message::Version("0.01");
		let msg = Message::Event(Event {
			typ: EventType::Word,
			start: 69,
			end: 420,
			name: Some("Hello :)"),
		});
		// buffer has fixed size in this case
		let mut buffer: [u8; 1024] = [0; 1024];
		let writer: OwnedFd = pipe_fd
			.try_into()
			.map_err(|_| Error::IOError("Cannot open file descriptor".to_string()))
			.expect("Unable to open file descriptor!");
		let mut file = PipeWriter::from(writer);
		// TODO: implement a more convenient way to not have to store a buffer, etc.
		let offset =
			write_message(&header, &mut buffer).expect("Unable to write to buffer!");
		let bytes_written_buf = write_message(&msg, &mut buffer[offset..])
			.expect("Unable to write to buffer!");
		let bytes_written_fd = file
			.write(&buffer[..bytes_written_buf + offset])
			.map_err(|_| Error::IOError("Cannot write to file descriptor".to_string()))
			.expect("Unable to write to file descriptor!");
		println!("Wrote {bytes_written_fd} bytes to Fd");
		assert_eq!(bytes_written_buf + offset, bytes_written_fd);
	}
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let voice = Voice {
		name: "My Voice".to_string(),
		id: "my-voice".to_string(),
		mime_format: "audio/x-spiel,format=S32LE,channels=1,rate=22050".to_string(),
		features: VoiceFeatureSet::empty(),
		// English, New Zealand
		languages: vec!["en-NZ".to_string()],
	};
	let voices = Vec::from([voice]);
	let provider = MySpeechProvider { voices };
	let _connection = Builder::session()?
		.name("org.domain.Speech.Provider")?
		.serve_at("/org/domain/Speech/Provider", provider)?
		.build()
		.await?;

	// wait forever for 60 seconds to test another program can receive the event!
	println!("Started provider! Go check if it works using the `test-provider` example!");
	sleep(Duration::from_secs(60)).await;
	Ok(())
}
