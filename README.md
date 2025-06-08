# `spiel`

Read and write audio, as well as mixed audio/events streams using the Spiel speech synthesis protocol.

## Cargo Features

Note that features with an unmarked checkbox are not yet implemented.

- [X] `default`: none. This includes all basic protocol functionality, both from bytes and into bytes: `no_std` and `no_alloc`. This feature set requires only `core`.
- [X] `client`: `std`, and pulls in the [`zbus`](https://crates.io/crates/zbus) crate. This provides a `Client` proxy type that ask for the speech provider to synthesize some speech, as well as query which voices and options are available.
- [X] `reader`: `alloc`. This gives you a sans-io `Reader` type where you can [`Reader::push`] bytes into the buffer, and then [`Reader::try_read`] to the conversion into a [`Message`].
    - This is _almost_ zero-copy. But currently requires a clone of the string if an event sent from the synthesizer has a name.
- [X] `alloc`: pulls in the [`bytes`](https://crates.io/crates/bytes), if `serde` is enabled. It exposes new types like [`crate::MessageOwned`] and [`crate::EventOwned`], which are owned versions of [`crate::Message`] and [`crate::Event`].
- [X] `poll`: add wrapper functions that return `Poll::Pending` when there is not enough data in the buffer. This is not for general use, but rather only if you are creating an async integration.
- [X] `serde`: activate [`serde::Serialize`] and [`serde::Deserialize`] on all types.
- [ ] `provider`: activates [`std`] and pulls in the [`zbus`](https://crates.io/crates/zbus) crate. This will provide the `SpeechProvider` struct, which can be used to provide speech over the Spiel protocol via `DBus`.

## MSRV

We use the [`str::from_utf8`] which was introduced in Rust `1.87`; with no features enabled, this is our MSRV.

With other features, YMMV.

## License

All contributions are dual-licensed under MIT or Apache-2.0.
_This crate is suitable for proprietary text-to-speech synthesizers wanting to be providers on the Spiel interface._

