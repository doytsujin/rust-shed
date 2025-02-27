/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

mod bytes_stream_future;

use std::cmp;
use std::io;
use std::io::BufRead;
use std::io::Read;

use bytes_old::BufMut;
use bytes_old::Bytes;
use bytes_old::BytesMut;
use futures::try_ready;
use futures::Async;
use futures::Poll;
use futures::Stream;
use tokio_io::codec::Decoder;
use tokio_io::AsyncRead;

pub use self::bytes_stream_future::BytesStreamFuture;

// 8KB is a reasonable default
const BUFSIZE: usize = 8 * 1024;

/// A structure that wraps a [Stream] of [Bytes] and lets it being accessed both
/// as a [Stream] and as [AsyncRead]. It is very useful when decoding Stream of
/// Bytes in an asynchronous way.
#[derive(Debug)]
pub struct BytesStream<S> {
    bytes: BytesMut,
    stream: S,
    stream_done: bool,
}

impl<S: Stream<Item = Bytes>> BytesStream<S> {
    /// Create a new instance of [BytesStream] wrapping the given [Stream] of [Bytes]
    pub fn new(stream: S) -> Self {
        BytesStream {
            bytes: BytesMut::with_capacity(BUFSIZE),
            stream,
            stream_done: false,
        }
    }

    /// Returns `true` if there are no more bytes left to be consumed
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty() && self.stream_done
    }

    /// Consumes this combinator returning a pair of bytes that have been received,
    /// but not yet consumed and the Stream that can possibly yield more bytes
    pub fn into_parts(self) -> (Bytes, S) {
        (self.bytes.freeze(), self.stream)
    }

    /// Returns a future that yields a single decoded item from the Bytes of this
    /// BytesStream (if any) and the remaining BytesStream.
    pub fn into_future_decode<Dec>(self, decoder: Dec) -> BytesStreamFuture<S, Dec>
    where
        Dec: Decoder,
        Dec::Error: From<S::Error>,
    {
        BytesStreamFuture::new(self, decoder)
    }

    /// Adds some bytes to the front of the BytesStream internal buffer. Those
    /// bytes are ready to be read immediately after this function completes.
    pub fn prepend_bytes(&mut self, bytes: Bytes) {
        let mut bytes_mut = match bytes.try_mut() {
            Ok(bytes_mut) => bytes_mut,
            Err(bytes) => {
                let cap = cmp::max(BUFSIZE, bytes.len() + self.bytes.len());
                let mut bytes_mut = BytesMut::with_capacity(cap);
                bytes_mut.put(bytes);
                bytes_mut
            }
        };

        bytes_mut.put(&self.bytes);
        self.bytes = bytes_mut;
    }

    fn poll_buffer(&mut self) -> Poll<(), S::Error> {
        if !self.stream_done {
            let bytes = try_ready!(self.stream.poll());
            match bytes {
                None => self.stream_done = true,
                Some(bytes) => self.bytes.extend_from_slice(&bytes),
            }
        }

        Ok(Async::Ready(()))
    }

    fn poll_buffer_until(&mut self, len: usize) -> Poll<(), S::Error> {
        while self.bytes.len() < len && !self.stream_done {
            try_ready!(self.poll_buffer());
        }

        Ok(Async::Ready(()))
    }
}

impl<S: Stream<Item = Bytes>> From<S> for BytesStream<S> {
    fn from(stream: S) -> Self {
        BytesStream::new(stream)
    }
}

impl<S> Read for BytesStream<S>
where
    S: Stream<Item = Bytes, Error = io::Error>,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let r#async = self.poll_buffer_until(buf.len())?;
        if self.bytes.is_empty() && r#async.is_not_ready() {
            Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "inner stream not ready",
            ))
        } else {
            let len = {
                let slice = self.bytes.as_ref();
                let len = cmp::min(buf.len(), slice.len());
                if len == 0 {
                    return Ok(0);
                }
                let slice = &slice[..len];
                let buf = &mut buf[..len];
                buf.copy_from_slice(slice);
                len
            };

            self.bytes.split_to(len);
            Ok(len)
        }
    }
}

impl<S> AsyncRead for BytesStream<S> where S: Stream<Item = Bytes, Error = io::Error> {}

impl<S> BufRead for BytesStream<S>
where
    S: Stream<Item = Bytes, Error = io::Error>,
{
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        if self.bytes.is_empty() && self.poll_buffer_until(1)?.is_not_ready() {
            Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "inner stream not ready",
            ))
        } else {
            Ok(self.bytes.as_ref())
        }
    }

    fn consume(&mut self, amt: usize) {
        self.bytes.split_to(amt);
    }
}

#[cfg(test)]
mod tests {
    use futures::stream::iter_ok;

    use super::*;
    use crate::BoxStream;
    use crate::StreamExt;

    fn make_reader(in_reads: Vec<Vec<u8>>) -> BytesStream<BoxStream<Bytes, io::Error>> {
        let stream = iter_ok(in_reads.into_iter().map(|v| v.into()));
        BytesStream::new(stream.boxify())
    }

    fn do_read<S>(reader: &mut BytesStream<S>, len_to_read: usize) -> io::Result<Vec<u8>>
    where
        S: Stream<Item = Bytes, Error = io::Error>,
    {
        let mut out = vec![0; len_to_read];
        let len_read = reader.read(&mut out)?;
        out.truncate(len_read);
        Ok(out)
    }

    #[test]
    fn test_read_once() -> io::Result<()> {
        let mut reader = make_reader(vec![vec![1, 2, 3, 4]]);
        let out = do_read(&mut reader, 4)?;
        assert_eq!(out, vec![1, 2, 3, 4]);
        Ok(())
    }

    #[test]
    fn test_read_join() -> io::Result<()> {
        let mut reader = make_reader(vec![vec![1, 2], vec![3, 4]]);
        let out = do_read(&mut reader, 4)?;
        assert_eq!(out, vec![1, 2, 3, 4]);
        Ok(())
    }

    #[test]
    fn test_read_split() -> io::Result<()> {
        let mut reader = make_reader(vec![vec![1, 2, 3, 4]]);
        let out = do_read(&mut reader, 2)?;
        assert_eq!(out, vec![1, 2]);
        let out = do_read(&mut reader, 2)?;
        assert_eq!(out, vec![3, 4]);
        Ok(())
    }

    #[test]
    fn test_read_eof() -> io::Result<()> {
        let mut reader = make_reader(vec![vec![1, 2, 3]]);
        let out = do_read(&mut reader, 4)?;
        assert_eq!(out, vec![1, 2, 3]);
        Ok(())
    }

    #[test]
    fn test_read_no_data() -> io::Result<()> {
        let mut reader = make_reader(vec![vec![1, 2, 3]]);
        let out = do_read(&mut reader, 4)?;
        assert_eq!(out, vec![1, 2, 3]);
        let out = do_read(&mut reader, 1)?;
        assert_eq!(out, vec![]);
        Ok(())
    }
}
