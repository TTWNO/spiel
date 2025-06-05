#![deny(
    clippy::pedantic,
    clippy::all,
    clippy::std_instead_of_core,
    clippy::std_instead_of_alloc,
    clippy::alloc_instead_of_core
)]
#![cfg_attr(not(feature = "std"), no_std)]

mod protocol;
#[cfg(feature = "reader")]
pub use protocol::Reader;
#[cfg(feature = "poll")]
pub use protocol::{poll_read_message_borrow, poll_read_message_type};
pub use protocol::{read_message_borrow, read_message_type, MessageBorrow, MessageType};
#[cfg(feature = "alloc")]
pub use protocol::{Event, Message};

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "client")]
pub mod client;

#[repr(u64)]
#[derive(Clone, Copy, Debug)]
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
