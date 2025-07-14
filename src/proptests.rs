use proptest::prelude::*;

use crate::{protocol::*, Reader, Writer};

// Strategy for EventType
impl Arbitrary for EventType {
	type Parameters = ();
	type Strategy = proptest::sample::Select<EventType>;

	fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
		proptest::sample::select(vec![
			EventType::Word,
			EventType::Sentence,
			EventType::Range,
			EventType::Mark,
		])
	}
}

// Strategy for ChunkType
impl Arbitrary for ChunkType {
	type Parameters = ();
	type Strategy = proptest::sample::Select<ChunkType>;

	fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
		proptest::sample::select(vec![ChunkType::Audio, ChunkType::Event])
	}
}

// Strategy for Event<'a>
impl Arbitrary for Event<'static> {
	type Parameters = ();
	type Strategy = BoxedStrategy<Event<'static>>;

	fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
		(
			any::<EventType>(),
			any::<u32>(),
			any::<u32>(),
			any::<Vec<char>>().prop_map(|chrs| {
				if chrs.is_empty() {
					None
				} else {
					Some(String::from_iter(&chrs[..]))
				}
			}),
		)
			.prop_map(|(typ, start, end, name)| Event {
				typ,
				start,
				end,
				name: if let Some(n) = name {
					Some(Box::leak(n.into_boxed_str()))
				} else {
					None
				},
			})
			.boxed()
	}
}

// Strategy for EventOwned
#[cfg(feature = "alloc")]
impl Arbitrary for EventOwned {
	type Parameters = ();
	type Strategy = BoxedStrategy<EventOwned>;

	fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
		(
			any::<EventType>(),
			any::<u32>(),
			any::<u32>(),
			any::<Vec<char>>().prop_map(|chrs| {
				if chrs.is_empty() {
					None
				} else {
					Some(String::from_iter(&chrs[..]))
				}
			}),
		)
			.prop_map(|(typ, start, end, name)| EventOwned { typ, start, end, name })
			.boxed()
	}
}

// Strategy for Message<'a>
impl Arbitrary for Message<'static> {
	type Parameters = ();
	type Strategy = BoxedStrategy<Message<'static>>;

	fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
		prop_oneof![
			// Audio
			proptest::collection::vec(any::<u8>(), 0..32)
				.prop_map(|v| Message::Audio(Box::leak(v.into_boxed_slice()))),
			// Event
			any::<Event<'static>>().prop_map(Message::Event),
		]
		.boxed()
	}
}

// Strategy for MessageOwned
#[cfg(feature = "alloc")]
impl Arbitrary for MessageOwned {
	type Parameters = ();
	type Strategy = BoxedStrategy<MessageOwned>;

	fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
		prop_oneof![
			".{4}".prop_map(MessageOwned::Version),
			proptest::collection::vec(any::<u8>(), 0..32)
				.prop_map(|v| MessageOwned::Audio(bytes::Bytes::from(v))),
			any::<EventOwned>().prop_map(MessageOwned::Event),
		]
		.boxed()
	}
}

// Strategy for MessageType
impl Arbitrary for MessageType {
	type Parameters = ();
	type Strategy = BoxedStrategy<MessageType>;

	fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
		prop_oneof![
			// Version
			proptest::collection::vec(any::<u8>(), 4).prop_map(|v| {
				let mut arr = [0; 4];
				for (i, c) in v.into_iter().enumerate().take(4) {
					arr[i] = c;
				}
				MessageType::Version { version: arr }
			}),
			// Audio
			(any::<usize>(), any::<usize>()).prop_map(
				|(samples_offset, samples_len)| MessageType::Audio {
					samples_offset,
					samples_len,
				}
			),
			// Event
			(
				any::<usize>(),
				any::<EventType>(),
				any::<u32>(),
				any::<u32>(),
				any::<usize>(),
			)
				.prop_map(|(name_offset, typ, start, end, name_len)| {
					MessageType::Event {
						name_offset,
						typ,
						start,
						end,
						name_len,
					}
				}),
		]
		.boxed()
	}
}

proptest::proptest! {
    #[test]
    fn message_roundtrip(
	msg in any::<Message>(),
    ) {
	let mut writer = Writer::new(Vec::new());
	writer.write_message(&msg)?;
	let mut reader = Reader::from(writer.inner);
	let header = reader.try_read()?;
	assert_eq!(header, Message::Version("0.01").into_owned());
	let decoded = reader.try_read()?;
	assert_eq!(msg.into_owned(), decoded);
    }
}

// TODO: an additional proptests for testing round-trips
#[cfg(feature = "alloc")]
proptest::proptest! {
    #[test]
    fn message_owned_roundtrip(msg in any::<Message>()) {
	let mut writer = Writer::new(Vec::new());
	writer.write_messages(&vec![msg.clone()][..]).expect("Unable to write message");
  writer.flush();
	let mut reader = Reader::from(writer.inner);
  let _version = reader.try_read()?;
	let decoded = reader.try_read()?;
	assert_eq!(msg.into_owned(), decoded);
    }
}
