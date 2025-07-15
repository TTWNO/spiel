use std::{io::PipeWriter, os::fd::OwnedFd, time::Duration};

use spiel::{Event, EventType, Message, Voice, VoiceFeatureSet, Writer};
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
		let msgs = [Message::Event(Event {
			typ: EventType::Word,
			start: 69,
			end: 420,
			name: Some("Hello :)"),
		})];
		// buffer has fixed size in this case
		let writer: OwnedFd = pipe_fd
			.try_into()
			.map_err(|_| Error::IOError("Cannot open file descriptor".to_string()))
			.expect("Unable to open file descriptor!");
		let file = PipeWriter::from(writer);
		let mut writer = Writer::new(file);
		writer.write_messages(&msgs).unwrap();
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
