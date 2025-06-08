//! This tool can be run with the `examples/provider.rs` binary to check the the service shows up
//! on DBus.
//! And that methods can be sent and dealt with appropriately.

use std::{error::Error, io, io::Read, os::fd::OwnedFd};

use spiel::{read_message, Client, Event, EventType, Message, Reader};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
	let client = Client::new().await?;
	let (mut reader, writer_pipe) = io::pipe()?;
	let writer = OwnedFd::from(writer_pipe);
	let providers = client.list_providers().await?;
	let mut found = false;
	for provider in providers {
		if provider.inner().destination() != "org.domain.Speech.Provider" {
			continue;
		}
		found = true;
		print!("TRY SEND...");
		provider.synthesize(
			writer.into(), // pipe writer
			"my-voice",    // voice ID
			"Hello!",      // text to synthesize
			0.5,           // pitch
			0.5,           // rate
			false,         // SSML on
			"en-NZ",       // English, New Zealand
		)
		.await?;
		println!("SENT!");
		let mut reader =
			Reader::from_source(reader).expect("Unable to create reader from pipe!");
		let header = reader.try_read().unwrap();
		assert_eq!(header, Message::Version("0.01").into_owned());
		println!("Received header information: {header:?}");
		let event = reader.try_read().unwrap();
		assert_eq!(
			event,
			Message::Event(Event {
				typ: EventType::Word,
				start: 69,
				end: 420,
				name: Some("Hello :)"),
			})
			.into_owned()
		);
		println!("Received event information: {event:?}");
		break;
	}
	assert!(found, "Could not find org.domain.Speech.Provider!");
	Ok(())
}
