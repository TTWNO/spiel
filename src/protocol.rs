//! Spiel protocol defenitions.

#[cfg(feature = "alloc")]
use alloc::{string::String, string::ToString};
#[cfg(feature = "poll")]
use core::task::Poll;
use core::{fmt, str::Utf8Error};

impl Message<'_> {
	/// Serializes the message into a Vec<u8> in the same binary format as the reader expects.
	#[cfg(feature = "alloc")]
	#[allow(clippy::cast_possible_truncation)]
	#[must_use]
	pub fn to_bytes(&self) -> alloc::vec::Vec<u8> {
		match self {
			Message::Version(version) => {
				let mut buf = alloc::vec::Vec::with_capacity(4);
				buf.extend_from_slice(version.as_bytes());
				buf
			}
			Message::Audio(samples) => {
				let mut buf = alloc::vec::Vec::with_capacity(1 + 4 + samples.len());
				buf.push(1); // ChunkType::Audio
				let len = samples.len() as u32;
				buf.extend_from_slice(&len.to_ne_bytes());
				buf.extend_from_slice(samples);
				buf
			}
			Message::Event(ev) => {
				let name_bytes = ev.name.map_or(&[][..], |n| n.as_bytes());
				let name_len = name_bytes.len() as u32;
				let mut buf = alloc::vec::Vec::with_capacity(
					1 + 1 + 4 + 4 + 4 + name_bytes.len(),
				);
				buf.push(2); // ChunkType::Event
				buf.push(ev.typ as u8);
				buf.extend_from_slice(&ev.start.to_ne_bytes());
				buf.extend_from_slice(&ev.end.to_ne_bytes());
				buf.extend_from_slice(&name_len.to_ne_bytes());
				buf.extend_from_slice(name_bytes);
				buf
			}
		}
	}
}

#[derive(Debug, Eq, PartialEq)]
pub enum Error {
	/// Reader does not have enough bytes to complete its read.
	/// Inner value specifies how many further bytes are needed.
	NotEnoughBytes(usize),
	/// Read an event type not specified in [`EventType`].
	InvalidEventType(u8),
	/// Read a chunk type not specified in [`ChunkType`].
	InvalidChunkType(u8),
	/// Writer does not have enough space to write into the buffer.
	/// Inner value specifies how many more bytes are necessary.
	NotEnoughSpace(usize),
	/// Unable to decode the str as utf8.
	/// Since the text should be ASCII conformant, this should never happen.
	Utf8(Utf8Error),
}
impl fmt::Display for Error {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Error::NotEnoughBytes(more) => {
				fmt.write_str("Not enough bytes; need ")?;
				more.fmt(fmt)?;
				fmt.write_str(" more")
			}
			Error::InvalidEventType(ty) => {
				fmt.write_str("Invalid event type: ")?;
				ty.fmt(fmt)?;
				fmt.write_str(". Valid values are 1, 2, 3, 4")
			}
			Error::InvalidChunkType(ty) => {
				fmt.write_str("Invalid chunk type: ")?;
				ty.fmt(fmt)?;
				fmt.write_str(". Valid values are 1, 2")
			}
			Error::NotEnoughSpace(more) => {
				fmt.write_str("Not enough space in write buffer; need ")?;
				more.fmt(fmt)?;
				fmt.write_str(" more")
			}
			Error::Utf8(utfe) => {
				fmt.write_str("UTF-8 Error: ")?;
				utfe.fmt(fmt)
			}
		}
	}
}
impl core::error::Error for Error {}

#[cfg(feature = "alloc")]
use bytes::Bytes;
#[cfg(feature = "reader")]
use bytes::BytesMut;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "alloc")]
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(all(feature = "serde", feature = "alloc"), derive(Serialize, Deserialize))]
pub struct EventOwned {
	pub typ: EventType,
	pub start: u32,
	pub end: u32,
	pub name: Option<String>,
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Event<'a> {
	pub typ: EventType,
	pub start: u32,
	pub end: u32,
	#[cfg_attr(feature = "serde", serde(borrow))]
	pub name: Option<&'a str>,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ChunkType {
	Event,
	Audio,
}

#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum EventType {
	Word = 1,
	Sentence = 2,
	Range = 3,
	Mark = 4,
}

impl EventType {
	fn to_ne_bytes(self) -> [u8; 1] {
		match self {
			EventType::Word => [1],
			EventType::Sentence => [2],
			EventType::Range => [3],
			EventType::Mark => [4],
		}
	}
}

fn read_version(buf: &[u8]) -> Result<(usize, Message<'_>), Error> {
	if buf.len() < 4 {
		return Err(Error::NotEnoughBytes(4 - buf.len()));
	}
	Ok((4, Message::Version(str::from_utf8(&buf[..4]).map_err(Error::Utf8)?)))
}
fn read_version_type(buf: &[u8]) -> Result<(usize, MessageType), Error> {
	if buf.len() < 4 {
		return Err(Error::NotEnoughBytes(4 - buf.len()));
	}
	let buf_4: &[u8; 4] = &buf[..4].try_into().expect("Exactly 4 bytes");
	Ok((4, MessageType::Version { version: *buf_4 }))
}

/// [`read_message`] takes a buffer and triees to read a [`Message`] from it.
/// This borrows data from the buffer, then returns a tuple containing:
///
/// 1. The number of bytes read.
/// 2. The borrowed message.
///
/// # Errors
///
/// - Not enough bytes in the buffer,
/// - Invalid variant of either [`ChunkType`] or [`EventType`],
/// - Converting a string into UTF-8 failed.
pub fn read_message(buf: &[u8], header_already_read: bool) -> Result<(usize, Message<'_>), Error> {
	if !header_already_read {
		return read_version(buf);
	}
	let (ct_offset, ct) = read_chunk_type(buf)?;
	let (offset, mt) = match ct {
		ChunkType::Audio => read_message_audio(&buf[ct_offset..]),
		ChunkType::Event => read_message_event(&buf[ct_offset..]),
	}?;
	Ok((ct_offset + offset, mt))
}

/// [`poll_read_message`] provides a method to _attempt_ to read Spiel messages from a buffer, but
/// mapped to a [`Poll`] type instead of a plain result.
/// See fields on [`MessageType`] to interpret the values that are returned in the happy-path case.
/// Unlike [`read_message_type`], this fails under only one condition, and in the case not enough data has been provided, it will return [`Poll::Pending`].
///
/// # Errors
///
/// Invalid data was provided in the buffer.
///
#[cfg(feature = "poll")]
#[allow(clippy::type_complexity)]
pub fn poll_read_message(
	buf: &[u8],
	header_already_read: bool,
) -> Poll<Result<(usize, Message<'_>), Error>> {
	match read_message(buf, header_already_read) {
		Ok(happy_data) => Poll::Ready(Ok(happy_data)),
		Err(Error::NotEnoughBytes(_)) => Poll::Pending,
		Err(e) => Poll::Ready(Err(e)),
	}
}

#[cfg(feature = "alloc")]
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(all(feature = "serde", feature = "alloc"), derive(Serialize, Deserialize))]
pub enum MessageOwned {
	Version(String),
	Audio(Bytes),
	Event(EventOwned),
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Message<'a> {
	#[cfg_attr(feature = "serde", serde(borrow))]
	Version(&'a str),
	#[cfg_attr(feature = "serde", serde(borrow))]
	Audio(&'a [u8]),
	#[cfg_attr(feature = "serde", serde(borrow))]
	Event(Event<'a>),
}

/// [`read_message_type`] allows you to get references to the input slice instead of placing it
/// into a [`Message`].
///
/// This is useful in the case that you are giving out references to the data on your stack.
/// Generally, you should use [`read_message`].
///
/// # Errors
///
/// - Not enough bytes in the buffer, or
/// - Invalid event variants.
pub fn read_message_type(
	buf: &[u8],
	header_already_read: bool,
) -> Result<(usize, MessageType), Error> {
	if !header_already_read {
		return read_version_type(buf);
	}
	let (ct_offset, ct) = read_chunk_type(buf)?;
	let (offset, msgt) = match ct {
		ChunkType::Audio => {
			let (cs_size, chunk_size) = read_u32(&buf[1..])?;
			let msg_b = MessageType::Audio {
				samples_offset: ct_offset + cs_size,
				samples_len: chunk_size as usize,
			};
			Ok((cs_size + chunk_size as usize, msg_b))
		}
		ChunkType::Event => {
			let (typ_len, typ) = read_event_type(&buf[ct_offset..])?;
			let (start_len, start) = read_u32(&buf[ct_offset + 1..])?;
			let (end_len, end) = read_u32(&buf[ct_offset + 5..])?;
			let (name_len_len, name_len) = read_u32(&buf[ct_offset + 9..])?;

			let msg_len =
				typ_len + start_len + end_len + name_len_len + name_len as usize;
			Ok((
				msg_len,
				MessageType::Event {
					typ,
					start,
					end,
					name_offset: ct_offset + 14,
					name_len: name_len as usize,
				},
			))
		}
	}?;
	Ok((offset + ct_offset, msgt))
}

#[cfg(feature = "alloc")]
impl Event<'_> {
	#[must_use]
	pub fn into_owned(self) -> EventOwned {
		EventOwned {
			typ: self.typ,
			start: self.start,
			end: self.end,
			name: self.name.map(ToString::to_string),
		}
	}
}

#[cfg(feature = "alloc")]
impl Message<'_> {
	#[must_use]
	pub fn into_owned(self) -> MessageOwned {
		match self {
			Message::Version(s) => MessageOwned::Version(s.to_string()),
			Message::Audio(frame) => MessageOwned::Audio(Bytes::copy_from_slice(frame)),
			Message::Event(ev) => MessageOwned::Event(ev.into_owned()),
		}
	}
}

fn read_u32(buf: &[u8]) -> Result<(usize, u32), Error> {
	if buf.len() < 4 {
		return Err(Error::NotEnoughBytes(4 - buf.len()));
	}
	let bytes: [u8; 4] = buf[..4].try_into().expect("at least 4 bytes");
	Ok((4, u32::from_ne_bytes(bytes)))
}
fn read_chunk_type(buf: &[u8]) -> Result<(usize, ChunkType), Error> {
	if buf.is_empty() {
		return Err(Error::NotEnoughBytes(1));
	}
	let ct = match buf[0] {
		1 => ChunkType::Audio,
		2 => ChunkType::Event,
		i => return Err(Error::InvalidChunkType(i)),
	};
	Ok((1, ct))
}

fn read_event_type(buf: &[u8]) -> Result<(usize, EventType), Error> {
	if buf.is_empty() {
		return Err(Error::NotEnoughBytes(1));
	}
	let et = match buf[0] {
		1 => EventType::Word,
		2 => EventType::Sentence,
		3 => EventType::Range,
		4 => EventType::Mark,
		i => return Err(Error::InvalidEventType(i)),
	};
	Ok((1, et))
}
fn read_message_audio(buf: &[u8]) -> Result<(usize, Message<'_>), Error> {
	let (cs_size, chunk_size) = read_u32(buf)?;
	let Some(audio_buf) = &buf.get(cs_size..(cs_size + chunk_size as usize)) else {
		return Err(Error::NotEnoughBytes((cs_size + chunk_size as usize) - buf.len()));
	};
	let msg_b = Message::Audio(audio_buf);
	Ok((cs_size + chunk_size as usize, msg_b))
}

fn read_message_event(buf: &[u8]) -> Result<(usize, Message<'_>), Error> {
	let (typ_len, typ) = read_event_type(buf)?;
	let (start_len, start) = read_u32(&buf[1..])?;
	let (end_len, end) = read_u32(&buf[5..])?;
	let (name_len_len, name_len) = read_u32(&buf[9..])?;

	let msg_len = typ_len + start_len + end_len + name_len_len + name_len as usize;
	let Some(name_buf) = &buf.get(13..(13 + name_len as usize)) else {
		return Err(Error::NotEnoughBytes((13 + name_len as usize) - buf.len()));
	};
	Ok((
		msg_len,
		Message::Event(Event {
			typ,
			start,
			end,
			name: if name_len == 0 {
				None
			} else {
				Some(str::from_utf8(name_buf).map_err(Error::Utf8)?)
			},
		}),
	))
}

#[test]
fn test_read_write_version() {
	let mt = Message::Version("wowz");
	let buf = &mut [0; 1024];
	let _offset = write_message(&mt.clone(), &mut buf[..]);
	let (_read_offset, mt2) = read_message(&buf[..], false).expect("Valid MessageType!");
	assert_eq!(mt, mt2);
}

#[test]
fn test_read_write_event() {
	let mt_name = "WTF is this!?";
	let mt = Message::Event(Event {
		typ: EventType::Word,
		start: 872,
		end: 99999,
		name: Some(mt_name),
	});
	let data = &mut [0u8; 1024];
	// No need to write this to the `data` buffer because we start with all 0s.
	let data_writer = &mut data[14..(14 + mt_name.len())];
	data_writer.copy_from_slice(mt_name.as_bytes());
	let buf = &mut [0u8; 1024];
	let _offset = write_message(&mt.clone(), &mut buf[..]);
	let (_read_offset, mt2) = read_message(&buf[..], true).expect("Valid MessageType!");
	assert_eq!(mt, mt2);
}

#[test]
fn test_read_write_audio() {
	let samples: [u8; 11] = [123, 93, 87, 16, 15, 15, 15, 0, 0, 0, 0];
	let mt = Message::Audio(&samples[..]);
	let data = &mut [0u8; 1024];
	let data_writer = &mut data[5..(5 + samples.len())];
	data_writer.copy_from_slice(&samples[..]);
	let buf = &mut [0u8; 1024];
	let _offset = write_message(&mt.clone(), &mut buf[..]);
	let (_read_offset, mt2) = read_message(&buf[..], true).expect("Valid MessageType");
	let Message::Audio(samples2) = mt2 else {
		assert_eq!(mt, mt2);
		panic!();
	};
	assert_eq!(&samples, &samples2);
	assert_eq!(mt, mt2);
}

/// Write a message to the buffer.
///
/// # Errors
///
/// Fails if the buiffer is too small.
pub fn write_message(mt: &Message, buf: &mut [u8]) -> Result<usize, Error> {
	match mt {
		Message::Version(version) => {
			if buf.len() < 4 {
				return Err(Error::NotEnoughSpace(4 - buf.len()));
			}
			let buf_first_4 = &mut buf[..4];

			buf_first_4.copy_from_slice(version.as_bytes());
			Ok(4)
		}
		Message::Audio(samples) => {
			if buf.len() < 5 + samples.len() {
				return Err(Error::NotEnoughSpace((5 + samples.len()) - buf.len()));
			}
			buf[0] = 1;
			let samples_len = samples.len();
			let samp_writer = &mut buf[1..5];
			#[allow(clippy::cast_possible_truncation)]
			samp_writer.copy_from_slice(&(samples_len as u32).to_ne_bytes());
			let data_writer = &mut buf[5..(samples_len + 5)];
			data_writer.copy_from_slice(samples);
			Ok(5 + samples_len)
		}
		Message::Event(Event { typ, start, end, name: maybe_name }) => {
			if buf.len() < 14 + maybe_name.unwrap_or_default().len() {
				return Err(Error::NotEnoughSpace(
					(14 + maybe_name.unwrap_or_default().len()) - buf.len(),
				));
			}
			buf[0] = 2;
			let typ_write = &mut buf[1..2];
			typ_write.copy_from_slice(&typ.to_ne_bytes());
			let start_write = &mut buf[2..6];
			start_write.copy_from_slice(&start.to_ne_bytes());
			let end_write = &mut buf[6..10];
			end_write.copy_from_slice(&end.to_ne_bytes());
			let name_len_write = &mut buf[10..14];
			let name_offset = 14;
			let name_len = maybe_name.unwrap_or_default().len();
			#[allow(clippy::cast_possible_truncation)]
			name_len_write.copy_from_slice(&(name_len as u32).to_ne_bytes());
			if let Some(name) = maybe_name {
				let name_write = &mut buf[name_offset..(name_offset + name_len)];
				name_write.copy_from_slice(name.as_bytes());
			}
			Ok(name_offset + name_len)
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
/// A type for interpreting buffer data, instead of taking references to the underlying data.
/// This is for when the data is on your stack, and you want to borrow the results yourself.
///
/// Mostly, you should use [`Message`] instead.
pub enum MessageType {
	Version {
		version: [u8; 4],
	},
	/// With this variant, you should then be able to:
	Audio {
		/// The index to the start of the data slice where the audio begins.
		samples_offset: usize,
		/// This length of the slice you should take in order to grab the audio frame.
		samples_len: usize,
	},
	Event {
		name_offset: usize,
		typ: EventType,
		start: u32,
		end: u32,
		name_len: usize,
	},
}

#[cfg(test)]
#[proptest::property_test]
fn never_panic(data: Vec<u8>, header: bool) {
	let mut new_data = &data[..];
	loop {
		if new_data.is_empty() {
			break;
		}
		let Ok((offset, _msg)) = read_message(new_data, header) else {
			break;
		};
		if let Some(next_data) = &new_data.get(offset..) {
			new_data = next_data;
		} else {
			break;
		}
	}
}
