use alloc::{string::ToString, vec::Vec};
#[cfg(feature = "std")]
use std::io;

use bytes::BytesMut;

use crate::{read_message_type, Error, EventOwned, MessageOwned, MessageType};

#[derive(Default)]
pub struct Reader {
	header_done: bool,
	buffer: BytesMut,
}

#[cfg(feature = "std")]
impl Reader {
	/// Uses a generic type that implements [`io::Read`] in order to construct the reader.
	/// This will call [`io::Read::read_to_end`] and will block the thread until complete.
	///
	/// # Errors
	///
	/// See [`io::Error`].
	pub fn from_source<T>(mut reader: T) -> Result<Self, io::Error>
	where
		T: io::Read,
	{
		let mut buffer_vec = Vec::new();
		reader.read_to_end(&mut buffer_vec)?;
		let mut buffer = BytesMut::new();
		buffer.extend_from_slice(&buffer_vec);
		Ok(Reader { header_done: false, buffer })
	}
}

impl From<Vec<u8>> for Reader {
	fn from(buf: Vec<u8>) -> Self {
		Reader { header_done: false, buffer: BytesMut::from(&buf[..]) }
	}
}

impl Reader {
	pub fn push(&mut self, other: &[u8]) {
		self.buffer.extend_from_slice(other);
	}
	/// Attempt to read from the reader's internal buffer.
	/// We further translate the data from [`MessageType`] into an owned [`Message`] for use.
	///
	/// # Errors
	///
	/// See [`read_message_type`] for failure cases.
	pub fn try_read(&mut self) -> Result<MessageOwned, Error> {
		let mut data = self.buffer.split().freeze();
		let (new_buf, message_type) = read_message_type(&data, self.header_done)
			.map(|(offset, mt)| (BytesMut::from(&data[offset..]), mt))?;

		let msg = match message_type {
			MessageType::Version { version } => {
				self.header_done = true;
				MessageOwned::Version(
					str::from_utf8(&version[..])
						.map_err(Error::Utf8)?
						.to_string(),
				)
			}
			MessageType::Audio { samples_offset, samples_len } => MessageOwned::Audio(
				data.split_off(samples_offset - 1).split_to(samples_len),
			),
			MessageType::Event { typ, start, end, name_offset, name_len } => {
				MessageOwned::Event(EventOwned {
					typ,
					start,
					end,
					name: if name_len == 0 {
						None
					} else {
						let bytes = data
							.split_off(name_offset - 1)
							.split_to(name_len);
						// TODO: try to remove this clone!
						let s = str::from_utf8(&bytes[..])
							.map_err(Error::Utf8)?
							.to_string();
						Some(s)
					},
				})
			}
		};

		self.buffer = new_buf;
		Ok(msg)
	}
}

#[test]
fn test_wave_reader() {
	use alloc::string::ToString;

	use assert_matches::assert_matches;

	use crate::EventType;
	let data: &[u8] = include_bytes!("../test.wav");
	let mut reader = Reader::from_source(data).expect("Able to make buffer from test.wav");
	assert_eq!(reader.try_read(), Ok(MessageOwned::Version("0.01".to_string())));
	assert_eq!(
		reader.try_read(),
		Ok(MessageOwned::Event(EventOwned {
			typ: EventType::Sentence,
			start: 0,
			end: 0,
			name: None
		}))
	);
	assert_eq!(
		reader.try_read(),
		Ok(MessageOwned::Event(EventOwned {
			typ: EventType::Word,
			start: 0,
			end: 4,
			name: None
		}))
	);
	for i in 0..4 {
		assert_matches!(reader.try_read(), Ok(MessageOwned::Audio(_)), "{i}");
	}
	let word_is = MessageOwned::Event(EventOwned {
		typ: EventType::Word,
		start: 5,
		end: 7,
		name: None,
	});
	let word_a = MessageOwned::Event(EventOwned {
		typ: EventType::Word,
		start: 8,
		end: 9,
		name: None,
	});
	let word_test = MessageOwned::Event(EventOwned {
		typ: EventType::Word,
		start: 10,
		end: 14,
		name: None,
	});
	let word_using = MessageOwned::Event(EventOwned {
		typ: EventType::Word,
		start: 15,
		end: 20,
		name: None,
	});
	let word_spiel = MessageOwned::Event(EventOwned {
		typ: EventType::Word,
		start: 21,
		end: 26,
		name: None,
	});
	let word_whaha = MessageOwned::Event(EventOwned {
		typ: EventType::Word,
		start: 28,
		end: 35,
		name: None,
	});
	assert_eq!(reader.try_read(), Ok(word_is));
	for _ in 0..3 {
		assert_matches!(reader.try_read(), Ok(MessageOwned::Audio(_)));
	}
	assert_eq!(reader.try_read(), Ok(word_a));
	assert_matches!(reader.try_read(), Ok(MessageOwned::Audio(_)));
	assert_matches!(reader.try_read(), Ok(MessageOwned::Audio(_)));
	assert_eq!(reader.try_read(), Ok(word_test));
	for _ in 0..6 {
		assert_matches!(reader.try_read(), Ok(MessageOwned::Audio(_)));
	}
	assert_eq!(reader.try_read(), Ok(word_using));
	for _ in 0..6 {
		assert_matches!(reader.try_read(), Ok(MessageOwned::Audio(_)));
	}
	assert_eq!(reader.try_read(), Ok(word_spiel));
	for _ in 0..14 {
		assert_matches!(reader.try_read(), Ok(MessageOwned::Audio(_)));
	}
	assert_eq!(
		reader.try_read(),
		Ok(MessageOwned::Event(EventOwned {
			typ: EventType::Sentence,
			start: 28,
			end: 28,
			name: None
		}))
	);
	assert_eq!(reader.try_read(), Ok(word_whaha));
	for _ in 0..10 {
		assert_matches!(reader.try_read(), Ok(MessageOwned::Audio(_)));
	}
	assert_eq!(&reader.buffer.freeze().slice(..)[..], &[]);
}
