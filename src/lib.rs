#![deny(
    clippy::pedantic,
    clippy::all,
)]

#[cfg(feature = "client")]
pub mod proxy;

use nom::{IResult,
    error::{Error, ErrorKind},
    bytes::streaming::take,
    combinator::{map, map_res, flat_map},
    Parser,
    number::{streaming::u32, Endianness},
    multi::length_data,
};

use serde::{Serialize, Deserialize};
use enumflags2::bitflags;

pub struct Reader {
    header_read: bool,
    buf: Vec<u8>,
}
impl Default for Reader {
    fn default() -> Self {
        Reader {
            header_read: false,
            buf: Vec::with_capacity(1024),
        }
    }
}

impl Reader {
    pub fn consume_bytes(&mut self, ext: &[u8]) {
        self.buf.extend_from_slice(ext);
    }
}

fn read_chunk(buf: &[u8], header_already_read: bool) -> IResult<&[u8], Chunk> {
	if !header_already_read {
		return map(
			read_header,
			|ver| Chunk::Version(ver),
		).parse(buf);
	}
  let (data,ct) = read_chunk_type(buf)?;
  match ct {
      ChunkType::Audio => {
          println!("Audio!");
          read_chunk_audio(data)
      },
      ChunkType::Event => {
          println!("Event!");
          read_chunk_event(data)
    },
  }
}

fn read_chunk_size(buf: &[u8]) -> IResult<&[u8], u32> {
    // TODO: should this always be native? I'm pretty sure it dpeneds on the stream parameters?
    u32(Endianness::Native)(buf)
}
fn read_header(buf: &[u8]) -> IResult<&[u8], &str> {
    println!("read_header");
    map_res(
        take(4usize),
        |bytes: &[u8]| {
            println!("BYTES: {:?}", bytes);
            core::str::from_utf8(bytes)
        }
    ).parse(buf)
}
fn read_chunk_type(buf: &[u8]) -> IResult<&[u8], ChunkType> {
    print!("chunk_type: ");
    map_res(take(1usize), |bytes: &[u8]| {
        println!("{:?}", bytes);
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
    buf: &'a [u8],
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

#[derive(Debug, PartialEq)]
pub enum Chunk<'a> {
    Version(&'a str),
    Audio(ChunkHold<'a>),
    Event(Event<'a>),
}

#[test]
fn test_wave() {
    use assert_matches::assert_matches;
    let data = include_bytes!("../test.wav");
    let (data, chunk) = read_chunk(data, false)
        .expect("read data!");
    assert_eq!(chunk, Chunk::Version("0.01"));
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_eq!(chunk, Chunk::Event(Event {
        typ: EventType::Sentence,
        start: 0,
        end: 0,
        name: None
    }));
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_eq!(chunk, Chunk::Event(Event {
        typ: EventType::Word,
        start: 0,
        end: 4,
        name: None
    }));
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, Chunk::Audio(_));
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, Chunk::Audio(_));
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, Chunk::Audio(_));
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, Chunk::Audio(_));
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
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
    assert_eq!(chunk, word_is);
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, Chunk::Audio(_));
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, Chunk::Audio(_));
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, Chunk::Audio(_));
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, word_is);
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, Chunk::Audio(_));
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, Chunk::Audio(_));
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, word_a);
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, Chunk::Audio(_));
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, Chunk::Audio(_));
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, Chunk::Audio(_));
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, Chunk::Audio(_));
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, Chunk::Audio(_));
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, Chunk::Audio(_));
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, word_test);
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, Chunk::Audio(_));
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, Chunk::Audio(_));
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, Chunk::Audio(_));
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, Chunk::Audio(_));
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, Chunk::Audio(_));
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, Chunk::Audio(_));
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, word_using);
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, Chunk::Audio(_));
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, Chunk::Audio(_));
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, Chunk::Audio(_));
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, Chunk::Audio(_));
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, Chunk::Audio(_));
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, Chunk::Audio(_));
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, Chunk::Audio(_));
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, Chunk::Audio(_));
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, Chunk::Audio(_));
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, Chunk::Audio(_));
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, Chunk::Audio(_));
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, Chunk::Audio(_));
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, Chunk::Audio(_));
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, Chunk::Audio(_));
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, word_using);
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, word_spiel);
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, Chunk::Audio(_));
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, Chunk::Audio(_));
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, Chunk::Audio(_));
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, Chunk::Audio(_));
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, Chunk::Audio(_));
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, Chunk::Audio(_));
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, Chunk::Audio(_));
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, Chunk::Audio(_));
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, Chunk::Audio(_));
    let (data, chunk) = read_chunk(data, true)
        .expect("read data!");
    assert_matches!(chunk, Chunk::Audio(_));
    assert_eq!(data, &[]);
}

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

#[bitflags]
#[repr(u64)]
#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
/// A bitfield of potential voice features that can be advertised to consumers.
pub enum VoiceFeature {
    /// Provider dispatches event when about to speak word.
    EventsWord,
    /// Provider dispatches event when about to speak sentence.
    EventsSentence,
    /// Provider dispatches event when about to speak unspecified range.
    EventsRange,
    /// Provider dispatches event when SSML mark is reached.
    EventsSsmlMark,
    /// </tp:docstring>
    SsmlSayAsDate,
    /// </tp:docstring>
    SsmlSayAsTime,
    /// </tp:docstring>
    SsmlSayAsTelephone,
    /// </tp:docstring>
    SsmlSayAsCharacters,
    /// </tp:docstring>
    SsmlSayAsCharactersGlyphs,
    /// </tp:docstring>
    SsmlSayAsCardinal,
    /// </tp:docstring>
    SsmlSayAsOrdinal,
    /// </tp:docstring>
    SsmlSayAsCurrency,
    /// </tp:docstring>
    SsmlBreak,
    /// </tp:docstring>
    SsmlSub,
    /// </tp:docstring>
    SsmlPhoneme,
    /// </tp:docstring>
    SsmlEmphasis,
    /// </tp:docstring>
    SsmlProsody,
    /// </tp:docstring>
    SsmlSentenceParagraph,
    /// </tp:docstring>
    SsmlToken,
}
