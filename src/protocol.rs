//! Spiel protocol defenitions.

use nom::{IResult,
    error::{Error, ErrorKind},
    bytes::streaming::take,
    combinator::{map, map_res},
    Parser,
    number::{streaming::u32, Endianness},
    multi::length_data,
    Input,
};
use bytes::{BytesMut, Bytes};
use alloc::string::String;

#[derive(Debug, PartialEq)]
pub struct Event {
    pub typ: EventType,
    pub start: u32,
    pub end: u32,
		pub name: Option<String>,
}

#[derive(Debug)]
pub enum ChunkType {
    Event,
    Audio
}

#[repr(u8)]
#[derive(Debug, PartialEq)]
pub enum EventType {
    Word,
    Sentence,
    Range,
    Mark,
}

#[derive(Debug)]
pub enum MessageType {
    Version { version: [char; 4] },
    /// With this variant, you should then be able to:
    Audio { 
        /// The index to the start of the data slice where the audio begins.
        samples_offset: usize,
        /// This length of the slice you should take in order to grab the audio frame.
        samples_len: usize },
    Event { name_offset: usize, typ: EventType, start: u32, end: u32, name_len: usize },
}
fn read_version(buf: &[u8]) -> IResult<&[u8], MessageType> {
    map(
        map(
            take(4usize),
            |bytes: &[u8]| {
                // SAFETY: This is allowed because we already know we've taken 4, and exactly 4
                // bytes at this point!
                [char::from(bytes[0]),
                char::from(bytes[1]),
                char::from(bytes[2]),
                char::from(bytes[3])]
            }
        ),
        |s| MessageType::Version { version: s }
    ).parse(buf)
}
pub fn read_message(buf: &[u8], header_already_read: bool) -> IResult<&[u8], MessageType> {
	if !header_already_read {
      return read_version(buf);
	}
  let (data,ct) = read_chunk_type(buf)?;
  match ct {
      ChunkType::Audio => {
          read_message_audio(data)
      },
      ChunkType::Event => {
          read_message_event(data)
    },
  }
}

#[derive(Debug, PartialEq)]
pub enum Message {
    Version(String),
    Audio(Bytes),
    Event(Event),
}

fn read_chunk_size(buf: &[u8]) -> IResult<&[u8], u32> {
    // TODO: should this always be native? I'm pretty sure it dpeneds on the stream parameters?
    u32(Endianness::Native)(buf)
}
fn read_chunk_type(buf: &[u8]) -> IResult<&[u8], ChunkType> {
    map_res(take(1usize), |bytes: &[u8]| {
        match bytes[0] {
            1 => Ok(ChunkType::Audio),
            2 => Ok(ChunkType::Event),
            _ => Err(nom::error::Error::new(bytes[0], ErrorKind::Tag))
        }
    }).parse(buf)
}

fn read_event_type(buf: &[u8]) -> IResult<&[u8], EventType> {
    map_res(take(1usize), |bytes: &[u8]| match bytes[0] {
        1 => Ok(EventType::Word),
        2 => Ok(EventType::Sentence),
        3 => Ok(EventType::Range),
        4 => Ok(EventType::Mark),
        _ => Err(Error::new(bytes[0], ErrorKind::Tag)),
    }).parse(buf)
}
fn read_message_audio(buf: &[u8]) -> IResult<&[u8], MessageType> {
    map(
        length_data(read_chunk_size),
        |data| MessageType::Audio { samples_offset: 5, samples_len: data.input_len() },
    ).parse(buf)
}

fn read_message_event(buf: &[u8]) -> IResult<&[u8], MessageType> {
    map((
        // Takes exactly 1 byte!!
        read_event_type,
        // then 3 x 4 bytes
        u32(Endianness::Native),
        u32(Endianness::Native),
        length_data(u32(Endianness::Native)),
        // ^ then take the number of bytes contianed in the data.
    ),
    |(typ,start,end,name_data)| MessageType::Event {
        name_offset: 14, typ, start, end, name_len: name_data.input_len(),
    }).parse(buf)
}

#[derive(Default)]
pub struct Reader {
    header_done: bool,
    buffer: BytesMut,
}
impl Reader {
    fn push(&mut self, other: &[u8]) {
        self.buffer.extend_from_slice(other)
    }
    fn try_read(&mut self) -> Result<Message, nom::Err<&[u8]>> {
        let mut data = self.buffer.split().freeze();
        let (new_buf, message_type) = match read_message(&data, self.header_done) {
            Ok((new_buf, message_type)) => {
                (BytesMut::from(new_buf.as_ref()), message_type)
            },
            Err(e) => {
                match e {
                    nom::Err::Incomplete(need) => panic!("NEED: {need:?}"),
                    nom::Err::Error(e) => panic!("ERROR: {:?}", e.code),
                    nom::Err::Failure(e) => panic!("ERROR: {:?}", e.code),
                }
            }
        };
        let msg = match message_type {
            MessageType::Version { version } => {
                self.header_done = true;
                Message::Version(String::from_iter(version.into_iter()))
            }
            MessageType::Audio { samples_offset, samples_len } => Message::Audio(
                data.split_off(samples_offset - 1).split_to(samples_len as usize)
            ),
            MessageType::Event { typ, start, end, name_offset, name_len } => Message::Event(Event {
                typ, start, end,
                name: if name_len == 0 { None } else {
                    // TODO: try to remove this clone!
                    Some(String::from_iter(data.split_off(name_offset - 1).split_to(name_len as usize).into_iter().map(char::from)))
                }
            }),
        };

        self.buffer = new_buf;
        Ok(msg)
    }
}

#[test]
fn test_wave_reader() {
    use assert_matches::assert_matches;
    use std::string::ToString;
    let mut reader = Reader::default();
    let mut data: &[u8] = include_bytes!("../test.wav");
    reader.push(data);
    assert_eq!(reader.try_read(), Ok(Message::Version("0.01".to_string())));
    assert_eq!(reader.try_read(), Ok(Message::Event(Event {
        typ: EventType::Sentence,
        start: 0,
        end: 0,
        name: None
    })));
    assert_eq!(reader.try_read(), Ok(Message::Event(Event {
        typ: EventType::Word,
        start: 0,
        end: 4,
        name: None
    })));
		for _ in 0..4 {
			assert_matches!(reader.try_read(), Ok(Message::Audio(_)));
		}
    let word_is = Message::Event(Event {
        typ: EventType::Word,
        start: 5,
        end: 7,
        name: None
    });
    let word_a = Message::Event(Event {
        typ: EventType::Word,
        start: 8,
        end: 9,
        name: None
    });
    let word_test = Message::Event(Event {
        typ: EventType::Word,
        start: 10,
        end: 14,
        name: None
    });
    let word_using = Message::Event(Event {
        typ: EventType::Word,
        start: 15,
        end: 20,
        name: None
    });
    let word_spiel = Message::Event(Event {
        typ: EventType::Word,
        start: 21,
        end: 27,
        name: None
    });
    let word_whaha = Message::Event(Event {
        typ: EventType::Word,
        start: 28,
        end: 36,
        name: None
    });
    assert_eq!(reader.try_read(), Ok(word_is));
		for _ in 0..3 {
			assert_matches!(reader.try_read(), Ok(Message::Audio(_)));
		}
    assert_matches!(reader.try_read(), Ok(word_is));
    assert_matches!(reader.try_read(), Ok(Message::Audio(_)));
    assert_matches!(reader.try_read(), Ok(Message::Audio(_)));
    assert_matches!(reader.try_read(), word_a);
		for _ in 0..6 {
			assert_matches!(reader.try_read(), Ok(Message::Audio(_)));
		}
    assert_matches!(reader.try_read(), word_test);
		for _ in 0..6 {
			assert_matches!(reader.try_read(), Ok(Message::Audio(_)));
		}
    assert_matches!(reader.try_read(), word_using);
		for _ in 0..14 {
			assert_matches!(reader.try_read(), Ok(Message::Audio(_)));
		}
    assert_matches!(reader.try_read(), word_using);
    assert_matches!(reader.try_read(), word_spiel);
		for _ in 0..10 {
			assert_matches!(reader.try_read(), Ok(Message::Audio(_)));
		}
    assert_eq!(&reader.buffer.freeze().slice(..)[..], &[]);
}
