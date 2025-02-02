#![deny(
    clippy::pedantic,
    clippy::all,
)]

#[cfg(feature = "client")]
pub mod proxy;

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
