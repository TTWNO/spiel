use nom::{IResult,
    error::{Error, ErrorKind},
    bytes::streaming::take,
    combinator::{map, map_res, flat_map},
    Parser,
    number::{streaming::u32, Endianness},
    multi::length_data,
};

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

#[derive(Debug, PartialEq)]
pub enum Chunk<'a> {
    Version(&'a str),
    Audio(ChunkHold<'a>),
    Event(Event<'a>),
}

pub fn read_chunk(buf: &[u8], header_already_read: bool) -> IResult<&[u8], Chunk> {
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
