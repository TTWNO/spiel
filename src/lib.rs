#![doc = include_str!("../README.md")]
#![deny(
	clippy::pedantic,
	clippy::all,
	clippy::std_instead_of_core,
	clippy::std_instead_of_alloc,
	clippy::alloc_instead_of_core,
	clippy::print_stdout,
	clippy::print_stderr,
	clippy::unwrap_used
)]
#![cfg_attr(not(any(feature = "std", test)), no_std)]

#[cfg(not(any(target_pointer_width = "64", target_pointer_width = "32")))]
compile_error!("You need at least 32-bit pointers to use this crate.");

mod protocol;
#[cfg(feature = "poll")]
pub use protocol::poll_read_message;
pub use protocol::{
	read_message, read_message_type, write_message, ChunkType, Error, Event, EventType,
	Message, MessageType,
};
#[cfg(feature = "alloc")]
pub use protocol::{EventOwned, MessageOwned};

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "client")]
pub mod client;
#[cfg(feature = "client")]
pub use client::{Client, Voice, VoiceFeatureSet};

#[cfg(feature = "reader")]
pub mod reader;
#[cfg(feature = "reader")]
pub use reader::Reader;

#[cfg(test)]
pub mod proptests;

#[cfg(feature = "std")]
pub mod writer;
#[cfg(feature = "std")]
pub use writer::Writer;
