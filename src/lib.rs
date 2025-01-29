#[cfg(feature = "client")]
pub mod proxy;

use serde::{Serialize, Deserialize};
use enumflags2::bitflags;

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
