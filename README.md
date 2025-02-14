# `spiel`

Read and write audio, and mixed audio/events using the Spiel speech synthesis protocol.

## Features

- [X] Parse byte stream (zero-copy, sans-io, zero-alloc).
- [X] Convert audio/events into bytes (zero-alloc).
- [X] Client proxy.
- [ ] Provider proxy.
- [ ] Protocol integrations.
    - [ ] tokio
    - [ ] async-std
    - [ ] smol-rs
    - [ ] embassy

## Cargo Features

- [X] `default-features`: none. This includes all protocol functionality, both from bytes and into bytes: `no_std`, `no_alloc`.
- [X] `provider`: `zbus`-based `Provider` proxy.
- [X] `client`: `zbus`-based `Client` proxy.
- [ ] `protocol-futures`: `Future`-based integrations for reading and writing using the protocol. This does not depend on any particular async library.
- [ ] `protocol-tokio`: `tokio` integrations for reading and writing using the protocol.
- [ ] `protocol-asyncstd`: `async-std` integrations for reading and writing using the protocol.
- [ ] `protocol-smol`: `smol-rs` integrations for reading and writing using the protocol.
- [ ] `protocol-embassy`: `embassy` integrations for reading and writing using the protocol.

## License

All contributions are dual-licensed under MIT or Apache-2.0.
_This crate is suitable for proprietary text-to-speech synthesizers wanting to be providers on the Spiel interface._

