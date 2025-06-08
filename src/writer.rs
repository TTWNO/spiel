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

	pub fn flush(&mut self) -> io::Result<()> {
		self.inner.flush()
	}
}
