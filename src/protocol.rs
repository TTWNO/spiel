//! Spiel protocol defenitions.

#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "poll")]
use core::task::Poll;

#[cfg(feature = "alloc")]
use bytes::Bytes;
#[cfg(feature = "reader")]
use bytes::BytesMut;
#[cfg(feature = "reader")]
use nom::Needed;
use nom::{
	bytes::streaming::take,
	combinator::{map, map_res},
	error::{Error, ErrorKind},
	multi::length_data,
	number::{streaming::u32, Endianness},
	IResult, Input, Parser,
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "alloc")]
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(all(feature = "serde", feature = "alloc"), derive(Serialize, Deserialize))]
pub struct Event {
	pub typ: EventType,
	pub start: u32,
	pub end: u32,
	pub name: Option<String>,
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct EventBorrow<'a> {
	pub typ: EventType,
	pub start: u32,
	pub end: u32,
	#[cfg_attr(feature = "serde", serde(borrow))]
	pub name: Option<&'a str>,
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ChunkType {
	Event,
	Audio,
}

#[repr(u8)]
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum EventType {
	Word = 1,
	Sentence = 2,
	Range = 3,
	Mark = 4,
}

impl EventType {
	fn to_ne_bytes(&self) -> [u8; 1] {
		match self {
			EventType::Word => [1],
			EventType::Sentence => [2],
			EventType::Range => [3],
			EventType::Mark => [4],
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum MessageType {
	Version {
		version: [char; 4],
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

impl MessageType {
	/// Determine how many bytes are necessary in order to write this (and its accompanying data
	/// slice) to a new buffer.
	fn bin_length(&self) -> usize {
		match self {
			// +1 for null byte
			// +1 for EventType::Event specifier
			// +4 for name_offset as u32
			MessageType::Event { name_len, .. } => name_len + 6,
			// +1 for EventType::Audio specifier
			// +4 for samples_offset as u32
			// +4 for start as u32
			// +4 for end as u32
			MessageType::Audio { samples_len, .. } => samples_len + 15,
			// 4 bytes for the version inforamtion; NO TERMINATING NULL BYTE!
			MessageType::Version { .. } => 4,
		}
	}
	/// Get the offset of the accompanying data in a buffer.
	fn offset(&self) -> usize {
		match self {
			MessageType::Version { .. } => 0,
			MessageType::Audio { samples_offset, .. } => *samples_offset,
			MessageType::Event { name_offset, .. } => *name_offset,
		}
	}
}

fn read_version(buf: &[u8]) -> IResult<&[u8], MessageType> {
	map(
		map(take(4usize), |bytes: &[u8]| {
			// SAFETY: This is allowed because we already know we've taken 4, and exactly 4
			// bytes at this point!
			[
				char::from(bytes[0]),
				char::from(bytes[1]),
				char::from(bytes[2]),
				char::from(bytes[3]),
			]
		}),
		|s| MessageType::Version { version: s },
	)
	.parse(buf)
}

fn read_version_borrow(buf: &[u8]) -> IResult<&[u8], MessageBorrow<'_>> {
	map(
		map_res(take(4usize), |bytes: &[u8]| str::from_utf8(&bytes[..4])),
		MessageBorrow::Version,
	)
	.parse(buf)
}
/// [`read_message_type`] provides a method to _attempt_ to read Spiel messages from a buffer.
/// See fields on [`MessageType`] to interpret the values that are returned in the happy-path case.
/// NOTE: you need to keep the buffer passed to this function alive in order to extract the audio
/// buffer.
///
/// ```no_run
/// use spiel::{
///     read_message_type,
///     MessageType,
/// };
/// use itertools::Itertools;
/// // Just imagine that you have mutable data from somewhere; this obviously won't run!
/// // See the [`filter_audio_data`] example to see how this is used in practice.
/// let mut data: &[u8] = &[0u8];
/// let mut header = false;
/// while let Ok((data_next, msg)) = read_message_type(data, header) {
///     header = true;
///     if let MessageType::Audio {
///         samples_offset,
///         samples_len,
///     } = msg
///     {
///         // NOTE: here is why the data must stay alive; `read_message_type` only gives you
///         // enough data to _reference_ the correct slices. It does not get these slices for you.
///         // If you want this functionality: receiving [`Message`] or [`MessageBorrow`]s
///         // directly, you want [`spiel::Reader`].
///         let ch = &data[samples_offset..samples_len];
///         for (l, h) in ch.iter().tuples() {
///             let sample = i16::from_le_bytes([*l, *h]);
///             // write this sample to a file/pipe/etc.
///         }
///     }
///     data = data_next;
/// }
/// ```
///
/// It fails under any condition which `nom` will fail to parse, but that usually comes down to 2
/// major cases:
///
/// # Errors
///
/// 1. Not enough data to parse a whole message (Error, recoverable, try again later)
/// 2. Invalid data (Failure, this means completely unrecoverable)
pub fn read_message_type(buf: &[u8], header_already_read: bool) -> IResult<&[u8], MessageType> {
	if !header_already_read {
		return read_version(buf);
	}
	let (data, ct) = read_chunk_type(buf)?;
	match ct {
		ChunkType::Audio => read_message_audio(data),
		ChunkType::Event => read_message_event(data),
	}
}

pub enum WriteError {
	NotEnoughSpaceInBuffer,
	DataNotLongEnough,
}

/// [`write_message_type`] will attempt to write a message's type and its data to a byte strim.
/// Returns the number of bytes written to the buffer upon success.
///
/// # Errors
///
/// - `data` buffer doesn't contain the data it's said to in `MessageType`
/// - `buf` buffer isn't long enough to take all the data.
///
pub fn write_message_type(
	mt: MessageType,
	data: &[u8],
	buf: &mut [u8],
) -> Result<usize, WriteError> {
	if mt.bin_length() > buf.len() {
		return Err(WriteError::NotEnoughSpaceInBuffer);
	}
	if mt.offset() + mt.bin_length() > data.len() {
		return Err(WriteError::DataNotLongEnough);
	}
	Ok(write_message_type_unchecked(mt, data, buf))
}

/// [`write_message_type_unchecked`] will write a message's type and its data to a byte stream.
///
/// # Returns
///
/// The number of bytes written.
///
/// # Panics
///
/// Panics if the space needed for writing the message is not large enough.
/// Or if the data slice does not contain enough data at the requested offset.
/// Use [`MessageType::bin_length`] to determine how long the buffer must be,
/// and use [`MessageType::offset`] to determine where in the data slice the accompanying data
/// should lie.
///
/// # SAFETY
///
/// Callers are required to either:
///
/// 1. Never give a partial `data` buffer (i.e., you may not continually pass
///    an offet further into the buffer upon successive calls). You *must* pass the entire data buffer
///    every time.
/// 2. Modify the offsets in the [`MessageType::Audio`] and [`MessageType::Event`] variants to
///    match the offsets read from the `data` buffer (the value returned from the function).
///
/// Use [`write_message_type`] if you want these failure cases handled for you.
pub fn write_message_type_unchecked(mt: MessageType, data: &[u8], buf: &mut [u8]) -> usize {
	match mt {
		MessageType::Version { version } => {
			let buf_first_4 = &mut buf[..4];
			// We could also have gotten it from the underlying data slice. Either way.
			let first_4_data = &[
				version[0] as u8,
				version[1] as u8,
				version[2] as u8,
				version[3] as u8,
			];
			buf_first_4.copy_from_slice(first_4_data);
			4
		}
		MessageType::Audio { samples_offset, samples_len } => {
			buf[0] = 1;
			let samp_writer = &mut buf[1..5];
			#[allow(clippy::cast_possible_truncation)]
			samp_writer.copy_from_slice(&(samples_len as u32).to_ne_bytes());
			let data_reader =
				&data[(5 + samples_offset)..(samples_offset + samples_len + 5)];
			let data_writer = &mut buf[5..(samples_len + 5)];
			data_writer.copy_from_slice(data_reader);
			5 + samples_len
		}
		MessageType::Event { typ, start, end, name_offset, name_len } => {
			buf[0] = 2;
			let typ_write = &mut buf[1..2];
			typ_write.copy_from_slice(&typ.to_ne_bytes());
			let start_write = &mut buf[2..6];
			start_write.copy_from_slice(&start.to_ne_bytes());
			let end_write = &mut buf[6..10];
			end_write.copy_from_slice(&end.to_ne_bytes());
			let name_len_write = &mut buf[10..14];
			#[allow(clippy::cast_possible_truncation)]
			name_len_write.copy_from_slice(&(name_offset as u32).to_ne_bytes());
			let name_write = &mut buf[14..(14 + name_len)];
			let name_read = &data[name_offset..(name_len + name_offset)];
			name_write.copy_from_slice(name_read);
			14 + name_len
		}
	}
}

/// `read_message_borrow` provides a method to _attempt_ to read Spiel messages from a buffer.
/// See fields on [`MessageBorrow`] to interpret the values that are returned in the happy-path case.
/// It fails under any condition which `nom` will fail to parse, but that usually comes down to 2
/// major cases:
///
/// # Errors
///
/// 1. Not enough data to parse a whole message (Error, recoverable, try again later)
/// 2. Invalid data (Failure, this means completely unrecoverable)
pub fn read_message_borrow(
	buf: &[u8],
	header_already_read: bool,
) -> IResult<&[u8], MessageBorrow<'_>> {
	if !header_already_read {
		return read_version_borrow(buf);
	}
	let (data, ct) = read_chunk_type(buf)?;
	match ct {
		ChunkType::Audio => read_message_borrow_audio(data),
		ChunkType::Event => read_message_borrow_event(data),
	}
}

/// [`poll_read_message_type`] provides a method to _attempt_ to read Spiel messages from a buffer, but
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
pub fn poll_read_message_type(
	buf: &[u8],
	header_already_read: bool,
) -> Poll<Result<(&[u8], MessageType), Error<&[u8]>>> {
	match read_message_type(buf, header_already_read) {
		Ok(happy_data) => Poll::Ready(Ok(happy_data)),
		Err(nom::Err::Incomplete(_)) => Poll::Pending,
		Err(nom::Err::Error(err) | nom::Err::Failure(err)) => Poll::Ready(Err(err)),
	}
}

/// `poll_read_message_borrow` provides a method to _attempt_ to read Spiel messages from a buffer, but
/// mapped to a [`Poll`] type instead of a plain result.
/// See fields on [`MessageType`] to interpret the values that are returned in the happy-path case.
/// Unline [`read_message_borrow`], this fails under only one condition, and in the case not enough data has been provided, it will return [`Poll::Pending`].
///
/// # Errors
///
/// Invalid data was provided in the buffer.
///
#[cfg(feature = "poll")]
#[allow(clippy::type_complexity)]
pub fn poll_read_message_borrow(
	buf: &[u8],
	header_already_read: bool,
) -> Poll<Result<(&[u8], MessageBorrow<'_>), Error<&[u8]>>> {
	match read_message_borrow(buf, header_already_read) {
		Ok(happy_data) => Poll::Ready(Ok(happy_data)),
		Err(nom::Err::Incomplete(_)) => Poll::Pending,
		Err(nom::Err::Error(err) | nom::Err::Failure(err)) => Poll::Ready(Err(err)),
	}
}

#[cfg(feature = "alloc")]
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(all(feature = "serde", feature = "alloc"), derive(Serialize, Deserialize))]
pub enum Message {
	Version(String),
	Audio(Bytes),
	Event(Event),
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum MessageBorrow<'a> {
	#[cfg_attr(feature = "serde", serde(borrow))]
	Version(&'a str),
	#[cfg_attr(feature = "serde", serde(borrow))]
	Audio(&'a [u8]),
	#[cfg_attr(feature = "serde", serde(borrow))]
	Event(EventBorrow<'a>),
}

fn read_chunk_size(buf: &[u8]) -> IResult<&[u8], u32> {
	// TODO: should this always be native? I'm pretty sure it dpeneds on the stream parameters?
	u32(Endianness::Native)(buf)
}
fn read_chunk_type(buf: &[u8]) -> IResult<&[u8], ChunkType> {
	map_res(take(1usize), |bytes: &[u8]| match bytes[0] {
		1 => Ok(ChunkType::Audio),
		2 => Ok(ChunkType::Event),
		_ => Err(nom::error::Error::new(bytes[0], ErrorKind::Tag)),
	})
	.parse(buf)
}

fn read_event_type(buf: &[u8]) -> IResult<&[u8], EventType> {
	map_res(take(1usize), |bytes: &[u8]| match bytes[0] {
		1 => Ok(EventType::Word),
		2 => Ok(EventType::Sentence),
		3 => Ok(EventType::Range),
		4 => Ok(EventType::Mark),
		_ => Err(Error::new(bytes[0], ErrorKind::Tag)),
	})
	.parse(buf)
}
fn read_message_audio(buf: &[u8]) -> IResult<&[u8], MessageType> {
	map(length_data(read_chunk_size), |data| MessageType::Audio {
		samples_offset: 5,
		samples_len: data.input_len(),
	})
	.parse(buf)
}
fn read_message_borrow_audio(buf: &[u8]) -> IResult<&[u8], MessageBorrow> {
	map(length_data(read_chunk_size), MessageBorrow::Audio).parse(buf)
}

fn read_message_event(buf: &[u8]) -> IResult<&[u8], MessageType> {
	map(
		(
			// Takes exactly 1 byte!!
			read_event_type,
			// then 3 x 4 bytes
			u32(Endianness::Native),
			u32(Endianness::Native),
			length_data(u32(Endianness::Native)),
			// ^ then take the number of bytes contianed in the data.
		),
		|(typ, start, end, name_data)| MessageType::Event {
			name_offset: 14,
			typ,
			start,
			end,
			name_len: name_data.input_len(),
		},
	)
	.parse(buf)
}
fn read_message_borrow_event(buf: &[u8]) -> IResult<&[u8], MessageBorrow<'_>> {
	map(
		(
			// Takes exactly 1 byte!!
			read_event_type,
			// then 3 x 4 bytes
			u32(Endianness::Native),
			u32(Endianness::Native),
			map_res(length_data(u32(Endianness::Native)), |bytes| {
				str::from_utf8(bytes)
			}),
		),
		|(typ, start, end, name)| {
			MessageBorrow::Event(EventBorrow {
				typ,
				start,
				end,
				name: if name.is_empty() { None } else { Some(name) },
			})
		},
	)
	.parse(buf)
}

#[cfg(feature = "reader")]
#[derive(Default)]
#[cfg_attr(all(feature = "serde", feature = "alloc"), derive(Serialize, Deserialize))]
pub struct Reader {
	header_done: bool,
	buffer: BytesMut,
}
#[cfg(feature = "reader")]
#[derive(Debug, PartialEq, Eq)]
/// TODO: Add `serde` support for this type!
pub enum ParseError {
	Needed(Needed),
	Error(Error<Bytes>),
	Failure(Error<Bytes>),
}

#[cfg(feature = "reader")]
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
	pub fn try_read(&mut self) -> Result<Message, ParseError> {
		let mut data = self.buffer.split().freeze();
		let (new_buf, message_type) = read_message_type(&data, self.header_done)
			.map(|(nb, mt)| (BytesMut::from(nb), mt))
			.map_err(|err| match err {
				nom::Err::Incomplete(need) => ParseError::Needed(need),
				nom::Err::Error(Error { input, code }) => {
					ParseError::Error(Error {
						code,
						input: data.slice(input.as_ptr().addr()
							- data.as_ptr().addr()..),
					})
				}
				nom::Err::Failure(Error { input, code }) => {
					ParseError::Failure(Error {
						code,
						input: data.slice(input.as_ptr().addr()
							- data.as_ptr().addr()..),
					})
				}
			})?;
		let msg = match message_type {
			MessageType::Version { version } => {
				self.header_done = true;
				Message::Version(version.into_iter().collect())
			}
			MessageType::Audio { samples_offset, samples_len } => Message::Audio(
				data.split_off(samples_offset - 1).split_to(samples_len),
			),
			MessageType::Event { typ, start, end, name_offset, name_len } => {
				Message::Event(Event {
					typ,
					start,
					end,
					name: if name_len == 0 {
						None
					} else {
						// TODO: try to remove this clone!
						Some(data
							.split_off(name_offset - 1)
							.split_to(name_len)
							.into_iter()
							.map(char::from)
							.collect::<String>())
					},
				})
			}
		};

		self.buffer = new_buf;
		Ok(msg)
	}
}

#[cfg(feature = "reader")]
#[test]
fn test_wave_reader() {
	use alloc::string::ToString;

	use assert_matches::assert_matches;
	let mut reader = Reader::default();
	let data: &[u8] = include_bytes!("../test.wav");
	reader.push(data);
	assert_eq!(reader.try_read(), Ok(Message::Version("0.01".to_string())));
	assert_eq!(
		reader.try_read(),
		Ok(Message::Event(Event {
			typ: EventType::Sentence,
			start: 0,
			end: 0,
			name: None
		}))
	);
	assert_eq!(
		reader.try_read(),
		Ok(Message::Event(Event { typ: EventType::Word, start: 0, end: 4, name: None }))
	);
	for _ in 0..4 {
		assert_matches!(reader.try_read(), Ok(Message::Audio(_)));
	}
	let word_is = Message::Event(Event { typ: EventType::Word, start: 5, end: 7, name: None });
	let word_a = Message::Event(Event { typ: EventType::Word, start: 8, end: 9, name: None });
	let word_test =
		Message::Event(Event { typ: EventType::Word, start: 10, end: 14, name: None });
	let word_using =
		Message::Event(Event { typ: EventType::Word, start: 15, end: 20, name: None });
	let word_spiel =
		Message::Event(Event { typ: EventType::Word, start: 21, end: 26, name: None });
	let word_whaha =
		Message::Event(Event { typ: EventType::Word, start: 28, end: 35, name: None });
	assert_eq!(reader.try_read(), Ok(word_is));
	for _ in 0..3 {
		assert_matches!(reader.try_read(), Ok(Message::Audio(_)));
	}
	assert_eq!(reader.try_read(), Ok(word_a));
	assert_matches!(reader.try_read(), Ok(Message::Audio(_)));
	assert_matches!(reader.try_read(), Ok(Message::Audio(_)));
	assert_eq!(reader.try_read(), Ok(word_test));
	for _ in 0..6 {
		assert_matches!(reader.try_read(), Ok(Message::Audio(_)));
	}
	assert_eq!(reader.try_read(), Ok(word_using));
	for _ in 0..6 {
		assert_matches!(reader.try_read(), Ok(Message::Audio(_)));
	}
	assert_eq!(reader.try_read(), Ok(word_spiel));
	for _ in 0..14 {
		assert_matches!(reader.try_read(), Ok(Message::Audio(_)));
	}
	assert_eq!(
		reader.try_read(),
		Ok(Message::Event(Event {
			typ: EventType::Sentence,
			start: 28,
			end: 28,
			name: None
		}))
	);
	assert_eq!(reader.try_read(), Ok(word_whaha));
	for _ in 0..10 {
		assert_matches!(reader.try_read(), Ok(Message::Audio(_)));
	}
	assert_eq!(&reader.buffer.freeze().slice(..)[..], &[]);
}

#[test]
fn test_read_write_version() {
	let mt = MessageType::Version { version: ['w', 'o', 'w', 'z'] };
	let data = &[0];
	let buf = &mut [0; 1024];
	let _offset = write_message_type_unchecked(mt.clone(), &data[..], &mut buf[..]);
	let (_read_offset, mt2) = read_message_type(&buf[..], false).expect("Valid MessageType!");
	assert_eq!(mt, mt2);
}

#[test]
fn test_read_write_event() {
	let mt_name = "WTF is this!?";
	let name_offset = 14;
	let mt = MessageType::Event {
		typ: EventType::Word,
		start: 872,
		end: 99999,
		name_offset,
		// +1: terminating null byte
		name_len: mt_name.len() + 1,
	};
	let data = &mut [0u8; 1024];
	// No need to write this to the `data` buffer because we start with all 0s.
	let data_writer = &mut data[14..(14 + mt_name.len())];
	data_writer.copy_from_slice(mt_name.as_bytes());
	let buf = &mut [0u8; 1024];
	let _offset = write_message_type_unchecked(mt.clone(), &data[..], &mut buf[..]);
	let (_read_offset, mt2) = read_message_type(&buf[..], true).expect("Valid MessageType!");
	let MessageType::Event { name_offset, name_len, .. } = mt2.clone() else {
		assert_eq!(mt, mt2);
		panic!();
	};
	// NOTE: the -1 is because the binary format requires the terminating null byte
	assert_eq!(&data[name_offset..(name_offset + name_len - 1)], mt_name.as_bytes());
	assert_eq!(mt, mt2);
}

#[test]
fn test_read_write_audio() {
	let samples = [123, 93, 87, 16, 15, 15, 15, 0, 0, 0, 0];
	let samples_offset = 5;
	let mt = MessageType::Audio { samples_offset, samples_len: samples.len() };
	let data = &mut [0u8; 1024];
	let data_writer = &mut data[5..(5 + samples.len())];
	data_writer.copy_from_slice(&samples[..]);
	let buf = &mut [0u8; 1024];
	let _offset = write_message_type_unchecked(mt.clone(), &data[..], &mut buf[..]);
	let (_read_offset, mt2) = read_message_type(&buf[..], true).expect("Valid MessageType!");
	let MessageType::Audio { samples_offset, samples_len } = mt2.clone() else {
		assert_eq!(mt, mt2);
		panic!();
	};
	assert_eq!(&data[samples_offset..(samples_offset + samples_len)], &samples);
	assert_eq!(mt, mt2);
}
