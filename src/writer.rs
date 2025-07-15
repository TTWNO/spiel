use std::io::{self, Write};

use crate::protocol::Message;

pub struct Writer<W: Write> {
	pub(crate) inner: W,
	header_done: bool,
	version: String,
}

impl<W: Write> Writer<W> {
	pub fn new(inner: W) -> Self {
		Writer { inner, version: "0.01".to_string(), header_done: false }
	}

	/// Write a single message into the buffer.
	///
	/// # Errors
	///
	/// See [`io::Error`].
	pub fn write_message(&mut self, message: &Message) -> Result<(), io::Error> {
		if !self.header_done {
			let header_msg = Message::Version(&self.version);
			let bytes = header_msg.to_bytes();
			self.inner.write_all(&bytes)?;
			self.header_done = true;
		}
		let bytes = message.to_bytes(); // Assuming Message has a to_bytes() method
		self.inner.write_all(&bytes)?;
		Ok(())
	}

	/// Write multiple messages into the buffer.
	///
	/// # Errors
	///
	/// See [`io::Error`].
	pub fn write_messages(&mut self, messages: &[Message]) -> Result<(), io::Error> {
		for message in messages {
			self.write_message(message)?;
		}
		Ok(())
	}

	/// Flush the buffer. Runs [`io::Writer::flush`].
	///
	/// # Errors
	///
	/// See [`io::Error`].
	pub fn flush(&mut self) -> io::Result<()> {
		self.inner.flush()
	}
}
