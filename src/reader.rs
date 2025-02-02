#[cfg(feature = "smol")]
use futures_lite::Stream;
#[cfg(any(feature = "smol"))]
use core::{
    task::{
        Poll,
        Context,
    },
    pin::{Pin, pin},
    marker::PhantomData,
};

use nom::{
    IResult,
    error::Error,
};
use alloc::{
    vec::Vec,
};
use crate::{
    Chunk,
    read_chunk,
};

pub struct Reader {
    read_header: bool,
    buf: Vec<u8>,
}

impl Default for Reader {
    fn default() -> Self {
        Reader {
            read_header: false,
            buf: Vec::with_capacity(1024),
        }
    }
}

impl Reader {
    /// Create a reader with a specific capacity.
    /// This will be automatically extended if the buffer is full.
    pub fn with_capacity(cap: usize) -> Self {
        Reader {
            read_header: false,
            buf: Vec::with_capacity(cap),
        }
    }
    /// Extend the internal buffer with the new slice.
    pub fn consume_bytes(&mut self, ext: &[u8]) {
        self.buf.extend_from_slice(ext);
    }
    /// Attempt to get the next item in the buffer.
    /// When this is done, you will get one of three return values.
    ///
    /// 1. Ok(Some(chunk)): this means a chunk was successfully read from the buffer.
    /// 2. Ok(None): this means that the parser was unable to _complete_ a chunk, but we may just
    ///    be waiting on more data.
    /// 3. Err(e): this means that there was an error reading from the bytestream. This is
    ///    unrecoverable. 
    pub fn try_read<'a, 'b>(&'a mut self) -> Result<Option<Chunk>, Error<&'b [u8]>> {
        let chunk = match read_chunk(&mut self.buf, self.read_header) {
            IResult::Ok((_, chunk)) => chunk,
            IResult::Err(nom::Err::Incomplete(_)) => return Ok(None),
            IResult::Err(nom::Err::Error(e)) => return Err(e),
            IResult::Err(nom::Err::Failure(e)) => return Err(e),
        };
        Ok(Some(chunk))
    }
}

#[cfg(feature = "smol")]
pub struct SmolReader<'a, 'b, S> 
where S: Stream<Item = &'a [u8]> {
    stream: S,
    reader: Reader,
    _marker: PhantomData<(&'a u8, &'b u8)>
}

#[cfg(feature = "smol")]
impl<'a, 'b, S> SmolReader<'a, 'b, S>
where S: Stream<Item = &'a [u8]> {
    fn from_stream(stream: S) -> Self {
        SmolReader {
            stream: stream,
            reader: Reader::default(),
            _marker: PhantomData,
        }
    }
}

#[cfg(feature = "smol")]
struct Smol;
/*
impl<'a, 'b, S> Stream for SmolReader<'a, 'b, S> 
where S: Stream<Item = &'a [u8]> + Unpin ,
'a: 'b {
    type Item = Result<Chunk<'b>, Error<&'b [u8]>>;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let stream = pin!(&mut self.stream);
        let data = match Stream::poll_next(stream, cx) {
            Poll::Ready(Some(data)) => data,
            Poll::Ready(None) |
            Poll::Pending => return Poll::Pending,
        };
        self.reader.consume_bytes(data);
        match self.reader.try_read() {
            Ok(Some(chunk)) => Poll::Ready(Some(Ok(chunk))),
            Ok(None) => Poll::Pending,
            Err(e) => Poll::Ready(Some(Err(e))),
        }
    }
}


*/
