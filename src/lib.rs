#![deny(
    clippy::pedantic,
    clippy::all,
)]

#[cfg(feature = "client")]
pub mod proxy;

use nom::{IResult,
    bytes::streaming::take,
    combinator::{map, map_res},
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

pub enum ChunkError {
    DecodeStr(core::str::Utf8Error),
		DecodeInt(core::num::TryFromIntError),
}
impl From<core::str::Utf8Error> for ChunkError {
    fn from(utfe: core::str::Utf8Error) -> ChunkError {
        ChunkError::DecodeStr(utfe)
    }
}
impl From<core::num::TryFromIntError> for ChunkError {
    fn from(utfe: core::num::TryFromIntError) -> ChunkError {
        ChunkError::DecodeInt(utfe)
    }
}

impl Reader {
    pub fn consume_bytes(&mut self, ext: &[u8]) {
        self.buf.extend_from_slice(ext);
    }
    /// Read existing bytes and create [`Event`]s from them.
    /// Ok(Some(Chunk)) will be returned when an [`Event`] can be parsed from the byte buffer.
    /// Otherwise (waiting for more bytes), Ok(None) will be returned.
    /// 
    /// # Errors
    ///
    /// - An invalid chunk identifier is used. [`enum@ChunkError::InvalidChunkValue`]. NOTE: since
    ///     chunk identifiers are necessary to determine the length of the chunk to read, this is an
    ///     unrecoverable error.
    pub fn read_bytes(&mut self) -> Result<Option<Chunk>, ChunkError> {
        if !self.header_read {
            return Ok(None);
        }
        Ok(None)
    }
}

fn read_chunk(buf: &[u8], header_already_read: bool) -> IResult<&[u8], Chunk> {
	if !header_already_read {
		return map_res(
			read_header,
			|ver| Ok::<Chunk, ChunkError>(Chunk::Version(ver)),
		).parse(buf);
	}
	todo!()
}

fn read_chunk_size(buf: &[u8]) -> IResult<&[u8], u32> {
    // TODO: should this always be native? I'm pretty sure it dpeneds on the stream parameters?
    u32(Endianness::Native)(buf)
}
fn read_header(buf: &[u8]) -> IResult<&[u8], &str> {
    map_res(take(4usize), |bytes: &[u8]| core::str::from_utf8(bytes)).parse(buf)
}
fn read_chunk_type(buf: &[u8]) -> IResult<&[u8], Option<ChunkType>> {
    map(take(1usize), |bytes: &[u8]| ChunkType::from_u8(bytes[0])).parse(buf)
}
fn read_event_type(buf: &[u8]) -> IResult<&[u8], Option<EventType>> {
    map(take(1usize), |bytes: &[u8]| EventType::from_u8(bytes[0])).parse(buf)
}
fn read_chunk_audio(buf: &[u8]) -> IResult<&[u8], ChunkHold> {
    map(
        length_data(read_chunk_size),
        |data| ChunkHold { buf: data },
    ).parse(buf)
}

pub struct ChunkHold<'a> {
    buf: &'a [u8],
}

fn read_event_data(buf: &[u8]) -> IResult<&[u8], Event> {
    map_res((
        u32(Endianness::Native),
        u32(Endianness::Native),
        length_data(
					u32::<&[u8], _>(Endianness::Native),
				),
    ),
    |(start,end,nme)| Ok::<Event, ChunkError>(Event { 
        start: start.try_into()?,
        end: end.try_into()?,
        name: if nme.len() > 0 {
					Some(core::str::from_utf8(nme)?) }else {None }
    }))
    .parse(buf)
}

pub enum Chunk<'a> {
    Version(&'a str),
    Audio(ChunkHold<'a>),
    Event(Event<'a>),
}

pub struct Event<'a> {
    pub start: usize,
    pub end: usize,
		pub name: Option<&'a str>,
}

pub enum ChunkType {
    Event,
    Audio
}
impl ChunkType {
    fn from_u8(b: u8) -> Option<Self> {
        match b {
            1 => Some(ChunkType::Event),
            2 => Some(ChunkType::Audio),
            _ => None,
        }
    }
}

#[repr(u8)]
pub enum EventType {
    Word,
    Sentence,
    Range,
    Mark,
}
impl EventType {
    fn from_u8(b: u8) -> Option<EventType> {
        match b {
            1 => Some(EventType::Word),
            2 => Some(EventType::Sentence),
            3 => Some(EventType::Range),
            4 => Some(EventType::Mark),
            _ => None
        }
    }
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
