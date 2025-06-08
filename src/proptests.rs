
use proptest::{prelude::*, string::bytes_regex};

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
			prop_oneof![Just(None), ".*".prop_map(Some),],
		)
			.prop_map(|(typ, start, end, name)| Event {
				typ,
				start,
				end,
				name: if name.is_none() {
					None
				} else {
					Some(Box::leak(name.unwrap().into_boxed_str()))
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
			prop_oneof![Just(None), ".*".prop_map(Some),],
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
			// Version
			//				bytes_regex(".{4}").unwrap()
			//            .prop_map(|s| Message::Version(&str::from_utf8(Box::leak(s.into_boxed_slice())).unwrap())),
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

fn bytes_ver() -> impl Strategy<Value = &'static str> {
	".*".prop_filter_map("Must be exactly 4 UTF-8 bytes", |s| {
		if s.len() == 4 {
			let leak: &'static str = Box::leak(s.into_boxed_str());
			Some(leak)
		} else {
			None
		}
	})
}

proptest::proptest! {
    #[test]
    fn message_roundtrip(
	msg in any::<Message>(),
	version in bytes_ver()
    ) {
	println!("VB: {version:?}");
	println!("VB+B: {:?}", version.bytes());
	let mut writer = Writer::new(Vec::new(), version);
	writer.write_message(&msg).unwrap();
	let mut reader = Reader::from(writer.inner);
	let header = reader.try_read()?;
	assert_eq!(header, Message::Version(&version).into_owned());
	let decoded = reader.try_read()?;
	assert_eq!(msg.into_owned(), decoded);
    }
}

#[cfg(feature = "alloc")]
proptest::proptest! {
    #[test]
    fn message_owned_roundtrip(msg in any::<MessageOwned>()) {
	//let mut writer = Writer::new(Vec::new());
	//writer.write_message(&msg).unwrap();
	//let mut reader = Reader::from(writer.inner);
	//let decoded = reader.try_read()?;
	//assert_eq!(msg, decoded);
    }
}
