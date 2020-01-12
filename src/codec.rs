use tokio_util::codec::Encoder;
use tokio_util::codec::Decoder;

use bytes::{Buf, BufMut, BytesMut};
use std::{cmp, fmt, io, str, usize};

use crate::wire::RawMsg;

/// A simple `Codec` implementation that splits up data into lines, and
/// the parses the result into RawMsg's
/// This is largely a complete copy-paste of upstream LinesCodec with 
/// minor changes at this point to auto-encode/decode into RawMsg
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct IrcCodec {
    // Stored index of the next index to examine for a `\n` character.
    // This is used to optimize searching.
    // For example, if `decode` was called with `abc`, it would hold `3`,
    // because that is the next index to examine.
    // The next time `decode` is called with `abcde\n`, the method will
    // only look at `de\n` before returning.
    next_index: usize,

    /// The maximum length for a given line. If `usize::MAX`, lines will be
    /// read until a `\n` character is reached.
    max_length: usize,

    /// Are we currently discarding the remainder of a line which was over
    /// the length limit?
    is_discarding: bool,
}

impl IrcCodec {
    /// Returns a `IrcCodec` for splitting up data into lines.
    ///
    /// # Note
    ///
    /// The returned `IrcCodec` will not have an upper bound on the length
    /// of a buffered line. See the documentation for [`new_with_max_length`]
    /// for information on why this could be a potential security risk.
    ///
    /// [`new_with_max_length`]: #method.new_with_max_length
    pub fn new() -> IrcCodec {
        IrcCodec {
            next_index: 0,
            max_length: 1024, // the IRC spec says 512, but I've doubled to 1024 for "non compliant" servers. If they exist?
            is_discarding: false,
        }
    }

    /// Returns the maximum line length when decoding.
    ///
    /// ```
    /// use std::usize;
    /// use tokio_util::codec::IrcCodec;
    ///
    /// let codec = IrcCodec::new();
    /// assert_eq!(codec.max_length(), usize::MAX);
    /// ```
    /// ```
    /// use tokio_util::codec::IrcCodec;
    ///
    /// let codec = IrcCodec::new_with_max_length(256);
    /// assert_eq!(codec.max_length(), 256);
    /// ```
    pub fn max_length(&self) -> usize {
        self.max_length
    }
}

fn utf8(buf: &[u8]) -> Result<&str, io::Error> {
    str::from_utf8(buf)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Unable to decode input as UTF8"))
}

fn without_carriage_return(s: &[u8]) -> &[u8] {
    if let Some(&b'\r') = s.last() {
        &s[..s.len() - 1]
    } else {
        s
    }
}

impl Decoder for IrcCodec {
    type Item = RawMsg;
    type Error = IrcCodecError;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<RawMsg>, IrcCodecError> {
        loop {
            // Determine how far into the buffer we'll search for a newline. If
            // there's no max_length set, we'll read to the end of the buffer.
            let read_to = cmp::min(self.max_length.saturating_add(1), buf.len());

            let newline_offset = buf[self.next_index..read_to]
                .iter()
                .position(|b| *b == b'\n');

            match (self.is_discarding, newline_offset) {
                (true, Some(offset)) => {
                    // If we found a newline, discard up to that offset and
                    // then stop discarding. On the next iteration, we'll try
                    // to read a line normally.
                    buf.advance(offset + self.next_index + 1);
                    self.is_discarding = false;
                    self.next_index = 0;
                }
                (true, None) => {
                    // Otherwise, we didn't find a newline, so we'll discard
                    // everything we read. On the next iteration, we'll continue
                    // discarding up to max_len bytes unless we find a newline.
                    buf.advance(read_to);
                    self.next_index = 0;
                    if buf.is_empty() {
                        return Err(IrcCodecError::MaxLineLengthExceeded);
                    }
                }
                (false, Some(offset)) => {
                    // Found a line!
                    let newline_index = offset + self.next_index;
                    self.next_index = 0;
                    let line = buf.split_to(newline_index + 1);
                    let line = &line[..line.len() - 1];
                    let line = without_carriage_return(line);
                    let line = utf8(line)?;

                    let msg = RawMsg::from_string(line.to_string());

                    //return Ok(Some(line.to_string()));
                    return Ok(Some(msg));
                }
                (false, None) if buf.len() > self.max_length => {
                    // Reached the maximum length without finding a
                    // newline, return an error and start discarding on the
                    // next call.
                    self.is_discarding = true;
                    return Err(IrcCodecError::MaxLineLengthExceeded);
                }
                (false, None) => {
                    // We didn't find a line or reach the length limit, so the next
                    // call will resume searching at the current offset.
                    self.next_index = read_to;
                    return Ok(None);
                }
            }
        }
    }

    fn decode_eof(&mut self, buf: &mut BytesMut) -> Result<Option<RawMsg>, IrcCodecError> {
        Ok(match self.decode(buf)? {
            Some(frame) => Some(frame),
            None => {
                // No terminating newline - return remaining data, if any
                if buf.is_empty() || buf == &b"\r"[..] {
                    None
                } else {
                    let line = buf.split_to(buf.len());
                    let line = without_carriage_return(&line);
                    let line = utf8(line)?;
                    self.next_index = 0;
                    //Some(line.to_string())

                    let msg = RawMsg::from_string(line.to_string());
                    Some(msg)
                }
            }
        })
    }
}

impl Encoder for IrcCodec {
    type Item = RawMsg;
    type Error = IrcCodecError;

    fn encode(&mut self, msg: RawMsg, buf: &mut BytesMut) -> Result<(), IrcCodecError> {
        let line = msg.to_string();

        println!("Sending: {}", line);

        buf.reserve(line.len() + 1);
        buf.put(line.as_bytes());
        buf.put_u8(b'\r');
        buf.put_u8(b'\n');
        Ok(())
    }
}

impl Default for IrcCodec {
    fn default() -> Self {
        Self::new()
    }
}

/// An error occured while encoding or decoding a line.
#[derive(Debug)]
pub enum IrcCodecError {
    /// The maximum line length was exceeded.
    MaxLineLengthExceeded,
    /// An IO error occured.
    Io(io::Error),
}

impl fmt::Display for IrcCodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IrcCodecError::MaxLineLengthExceeded => write!(f, "max line length exceeded"),
            IrcCodecError::Io(e) => write!(f, "{}", e),
        }
    }
}

impl From<io::Error> for IrcCodecError {
    fn from(e: io::Error) -> IrcCodecError {
        IrcCodecError::Io(e)
    }
}

impl std::error::Error for IrcCodecError {}
