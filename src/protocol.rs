//! Spiel protocol defenitions.

use nom::{IResult,
    Finish,
    error::{Error, ErrorKind},
    bytes::streaming::take,
    combinator::{map, map_res, flat_map},
    Parser,
    number::{streaming::u32, Endianness},
    multi::length_data,
};
use bytes::Buf;
use core::task::Poll;
use core::borrow::Borrow;
use core::iter::once;
use alloc::vec::Vec;

#[test]
fn test_version_string() {
   let data = "6.90";
   let (_, chunk) = read_chunk(data.as_bytes(), false)
    .expect("read data!");
   assert_eq!(chunk, Chunk::Version("6.90"));
}

#[derive(Debug, PartialEq)]
pub struct Event<'a> {
    pub typ: EventType,
    pub start: u32,
    pub end: u32,
		pub name: Option<&'a str>,
}
impl<'a> Event<'a> {
    fn to_bytes(self) -> ([u8; 13], &'a [u8]) {
        let start_bytes = self.start.to_le_bytes();
        let end_bytes = self.end.to_le_bytes();
        let name_len_bytes = self.name.unwrap_or_default().len().to_le_bytes();
        let name_size = self.name.unwrap().len();
        let discriminant: u8 = match self.typ {
            EventType::Word => 1,
            EventType::Sentence => 2,
            EventType::Range => 3,
            EventType::Mark => 4,
        };
        let slice = [
            discriminant,
            start_bytes[0],
            start_bytes[1],
            start_bytes[2],
            start_bytes[3],
            end_bytes[0],
            end_bytes[1],
            end_bytes[2],
            end_bytes[3],
            name_len_bytes[0],
            name_len_bytes[1],
            name_len_bytes[2],
            name_len_bytes[3],
        ];
        let name_bytes = match self.name {
            Some(name) => name.as_bytes(),
            None => &[],
        };
        (slice, name_bytes)
    }
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

#[derive(Debug, PartialEq)]
pub enum Chunk<'a> {
    Version(&'a str),
    Audio(ChunkHold<'a>),
    Event(Event<'a>),
}

impl<'a> Chunk<'a> {
    fn into_bytes(self) -> &'a [u8] {
        match self {
            Chunk::Version(version) => version.as_bytes(),
            Chunk::Audio(ch) => ch.buf,
            Chunk::Event(ev) => {
                let (start, end) = ev.to_bytes();
                todo!()
            }
        }
    }
}

pub fn read_chunk(buf: &[u8], header_already_read: bool) -> IResult<&[u8], Chunk> {
	if !header_already_read {
      return read_header(buf);
	}
  let (data,ct) = read_chunk_type(buf)?;
  match ct {
      ChunkType::Audio => {
          read_chunk_audio(data)
      },
      ChunkType::Event => {
          read_chunk_event(data)
    },
  }
}

fn read_chunk_size(buf: &[u8]) -> IResult<&[u8], u32> {
    // TODO: should this always be native? I'm pretty sure it dpeneds on the stream parameters?
    u32(Endianness::Native)(buf)
}
fn read_header(buf: &[u8]) -> IResult<&[u8], Chunk> {
    map(
        map_res(
            take(4usize),
            |bytes: &[u8]| {
                core::str::from_utf8(bytes)
            }
        ),
        |s| Chunk::Version(s)
    ).parse(buf)
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
fn read_chunk_audio(buf: &[u8]) -> IResult<&[u8], Chunk> {
    map(
        length_data(read_chunk_size),
        |data| Chunk::Audio(ChunkHold { buf: data }),
    ).parse(buf)
}

#[derive(PartialEq)]
pub struct ChunkHold<'a> {
    pub buf: &'a [u8],
}
impl core::fmt::Debug for ChunkHold<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ChunkHold")
         .field("len", &self.buf.len())
         .finish()
    }
}

fn read_chunk_event(buf: &[u8]) -> IResult<&[u8], Chunk> {
    map((
        read_event_type,
        u32(Endianness::Native),
        u32(Endianness::Native),
        map_res(
            length_data(
              u32::<&[u8], _>(Endianness::Native),
            ),
            |bytes| core::str::from_utf8(bytes)
        ),
    ),
    |(typ,start,end,name)| Chunk::Event(Event {
        typ, start, end, name: if name.len() > 0 { Some(name) } else { None }
    })).parse(buf)
}

pub fn poll_read_chunk(buf: &[u8], read_header: bool) -> Poll<IResult<&[u8], Chunk<'_>>> {
    match read_chunk(buf, read_header) {
        Ok(buf_and_chunk) => Poll::Ready(Ok(buf_and_chunk)),
        Err(e) => Poll::Ready(Err(e)),
        Err(nom::Err::Incomplete(_)) => Poll::Pending,
    }
}

#[derive(Default)]
pub struct Reader {
    header_done: bool,
    buffer: bytes::BytesMut,
}
impl Reader {
    fn push(&mut self, other: &[u8]) {
        self.buffer.extend_from_slice(other)
    }
    fn try_read(&mut self) -> Result<Chunk<'_>, nom::Err<&[u8]>> {
        let data = self.buffer.split().freeze();
        let (new_buf, chunk) = match read_chunk(&data, self.header_done) {
            Ok((new_buf, chunk)) => {
                let size = data.len() - new_buf.len();
                (bytes::BytesMut::from(new_buf.as_ref()), chunk)
            },
            _ => panic!(),
        };
        let size = self.buffer.len() - new_buf.len();
        self.buffer.advance(size);
        Ok(chunk)
    }
}

#[test]
fn test_wave() {
    use assert_matches::assert_matches;
    let mut data: &[u8] = include_bytes!("../test.wav");
		let mut chunk = Chunk::Version("");
    (data, chunk) = read_chunk(data, false)
        .expect("read data!");
    assert_eq!(chunk, Chunk::Version("0.01"));
    (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_eq!(chunk, Chunk::Event(Event {
        typ: EventType::Sentence,
        start: 0,
        end: 0,
        name: None
    }));
    (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_eq!(chunk, Chunk::Event(Event {
        typ: EventType::Word,
        start: 0,
        end: 4,
        name: None
    }));
		for _ in 0..4 {
			(data, chunk) = read_chunk(data, true)
					.expect("read data!");
			assert_matches!(chunk, Chunk::Audio(_));
		}
    let word_is = Chunk::Event(Event {
        typ: EventType::Word,
        start: 5,
        end: 7,
        name: None
    });
    let word_a = Chunk::Event(Event {
        typ: EventType::Word,
        start: 8,
        end: 9,
        name: None
    });
    let word_test = Chunk::Event(Event {
        typ: EventType::Word,
        start: 10,
        end: 14,
        name: None
    });
    let word_using = Chunk::Event(Event {
        typ: EventType::Word,
        start: 15,
        end: 20,
        name: None
    });
    let word_spiel = Chunk::Event(Event {
        typ: EventType::Word,
        start: 21,
        end: 27,
        name: None
    });
    let word_whaha = Chunk::Event(Event {
        typ: EventType::Word,
        start: 28,
        end: 36,
        name: None
    });
    (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_eq!(chunk, word_is);
		for _ in 0..3 {
			(data, chunk) = read_chunk(data, true)
					.expect("read data!");
			assert_matches!(chunk, Chunk::Audio(_));
		}
    (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, word_is);
    (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, Chunk::Audio(_));
    (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, Chunk::Audio(_));
    (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, word_a);
		for _ in 0..6 {
			(data, chunk) = read_chunk(data, true)
					.expect("read data!");
			assert_matches!(chunk, Chunk::Audio(_));
		}
    (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, word_test);
		for _ in 0..6 {
			(data, chunk) = read_chunk(data, true)
					.expect("read data!");
			assert_matches!(chunk, Chunk::Audio(_));
		}
    (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, word_using);
		for _ in 0..14 {
			(data, chunk) = read_chunk(data, true)
					.expect("read data!");
			assert_matches!(chunk, Chunk::Audio(_));
		}
    (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, word_using);
    (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, word_spiel);
		for _ in 0..10 {
			(data, chunk) = read_chunk(data, true)
					.expect("read data!");
			assert_matches!(chunk, Chunk::Audio(_));
		}
    assert_eq!(data, &[]);
}
