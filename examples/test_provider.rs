//! This tool can be run with the `examples/provider.rs` binary to check the the service shows up
//! on DBus.
//! And that methods can be sent and dealt with appropriately.

use std::{error::Error, io, io::Read, os::fd::OwnedFd};

use spiel::{read_message, Client, Event, EventType, Message};

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
		let mut buf = Vec::new();
		let bytes_read = reader.read_to_end(&mut buf)?;
		println!("BYTES READ: {bytes_read}");
		let (bytes_read2, header) = read_message(&buf[..], false)?;
		let (bytes_read3, msg) = read_message(&buf[bytes_read2..], true)?;
		assert_eq!(bytes_read, bytes_read2 + bytes_read3);
		assert_eq!(header, Message::Version("0.01"));
		assert_eq!(
			msg,
			Message::Event(Event {
				typ: EventType::Word,
				start: 69,
				end: 420,
				name: Some("Hello :)"),
			})
		);
		break;
	}
	assert!(found, "Could not find org.domain.Speech.Provider!");
	Ok(())
}
