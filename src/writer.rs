use std::io::{self, Write};

use crate::protocol::{Error, Message};

pub struct Writer<W: Write> {
	pub(crate) inner: W,
	header_done: bool,
	version: String,
}

impl<W: Write> Writer<W> {
	pub fn new(inner: W, version: &str) -> Self {
		Writer { inner, version: version.to_string(), header_done: false }
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
		}
		let bytes = message.to_bytes(); // Assuming Message has a to_bytes() method
		self.inner.write_all(&bytes)?;
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
