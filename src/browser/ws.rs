//! The smallest WebSocket client that can hold a conversation with
//! Chrome.
//!
//! The DevTools Protocol is JSON over a WebSocket, and that is the only
//! reason this exists. It speaks to `127.0.0.1` over plain TCP, to a
//! process this build started itself, so it needs none of what a general
//! client needs: no TLS, no permessage-deflate, no fragmentation to send,
//! no ping policy beyond answering one. What is left is the handshake and
//! the frame, and those are small enough to write down — the same
//! judgement that keeps the PDF writer and the gzip encoder here.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{Result, WebFluentError};

fn oops(what: impl std::fmt::Display) -> WebFluentError {
    WebFluentError::IoError(format!("browser: {what}"))
}

/// An open connection to a DevTools endpoint.
pub struct Socket {
    stream: TcpStream,
    reader: BufReader<TcpStream>,
    seed: u64,
}

impl Socket {
    /// Connect to `ws://host:port/path`.
    pub fn connect(url: &str) -> Result<Self> {
        let rest = url
            .strip_prefix("ws://")
            .ok_or_else(|| oops(format!("not a websocket address: {url}")))?;
        let (authority, path) = match rest.find('/') {
            Some(at) => (&rest[..at], &rest[at..]),
            None => (rest, "/"),
        };
        let stream = TcpStream::connect(authority).map_err(oops)?;
        stream.set_nodelay(true).ok();
        let reader = BufReader::new(stream.try_clone().map_err(oops)?);
        let mut socket = Socket {
            stream,
            reader,
            seed: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(0x2545F4914F6CDD1D),
        };
        socket.handshake(authority, path)?;
        Ok(socket)
    }

    fn handshake(&mut self, authority: &str, path: &str) -> Result<()> {
        let key = base64(&self.bytes(16));
        let request = format!(
            "GET {path} HTTP/1.1\r\n\
             Host: {authority}\r\n\
             Upgrade: websocket\r\n\
             Connection: Upgrade\r\n\
             Sec-WebSocket-Key: {key}\r\n\
             Sec-WebSocket-Version: 13\r\n\r\n"
        );
        self.stream.write_all(request.as_bytes()).map_err(oops)?;
        let mut status = String::new();
        self.reader.read_line(&mut status).map_err(oops)?;
        if !status.contains("101") {
            return Err(oops(format!(
                "the endpoint refused the upgrade: {}",
                status.trim()
            )));
        }
        loop {
            let mut line = String::new();
            let read = self.reader.read_line(&mut line).map_err(oops)?;
            if read == 0 || line.trim().is_empty() {
                break;
            }
        }
        Ok(())
    }

    /// Send one text message. A client's frame is always masked.
    pub fn send(&mut self, text: &str) -> Result<()> {
        let payload = text.as_bytes();
        let mut frame = vec![0x81u8]; // FIN + text
        let mask_bit = 0x80u8;
        match payload.len() {
            n if n < 126 => frame.push(mask_bit | n as u8),
            n if n <= u16::MAX as usize => {
                frame.push(mask_bit | 126);
                frame.extend_from_slice(&(n as u16).to_be_bytes());
            }
            n => {
                frame.push(mask_bit | 127);
                frame.extend_from_slice(&(n as u64).to_be_bytes());
            }
        }
        let mask = self.bytes(4);
        frame.extend_from_slice(&mask);
        frame.extend(payload.iter().enumerate().map(|(i, b)| b ^ mask[i % 4]));
        self.stream.write_all(&frame).map_err(oops)?;
        self.stream.flush().map_err(oops)
    }

    /// The next text message, answering a ping and stepping over
    /// everything else on the way.
    pub fn receive(&mut self) -> Result<String> {
        loop {
            let mut head = [0u8; 2];
            self.reader.read_exact(&mut head).map_err(oops)?;
            let opcode = head[0] & 0x0F;
            let masked = head[1] & 0x80 != 0;
            let length = match head[1] & 0x7F {
                126 => {
                    let mut n = [0u8; 2];
                    self.reader.read_exact(&mut n).map_err(oops)?;
                    u16::from_be_bytes(n) as usize
                }
                127 => {
                    let mut n = [0u8; 8];
                    self.reader.read_exact(&mut n).map_err(oops)?;
                    u64::from_be_bytes(n) as usize
                }
                n => n as usize,
            };
            let mask = if masked {
                let mut m = [0u8; 4];
                self.reader.read_exact(&mut m).map_err(oops)?;
                Some(m)
            } else {
                None
            };
            let mut payload = vec![0u8; length];
            self.reader.read_exact(&mut payload).map_err(oops)?;
            if let Some(mask) = mask {
                for (i, byte) in payload.iter_mut().enumerate() {
                    *byte ^= mask[i % 4];
                }
            }
            match opcode {
                0x1 => return String::from_utf8(payload).map_err(oops),
                // A ping wants its body back; a close ends the conversation.
                0x9 => self.pong(&payload)?,
                0x8 => return Err(oops("the browser closed the connection")),
                _ => continue,
            }
        }
    }

    fn pong(&mut self, body: &[u8]) -> Result<()> {
        let mask = self.bytes(4);
        let mut frame = vec![0x8Au8, 0x80 | body.len() as u8];
        frame.extend_from_slice(&mask);
        frame.extend(body.iter().enumerate().map(|(i, b)| b ^ mask[i % 4]));
        self.stream.write_all(&frame).map_err(oops)?;
        self.stream.flush().map_err(oops)
    }

    /// How long to wait for a message before giving up.
    pub fn deadline(&self, seconds: u64) -> Result<()> {
        self.stream
            .set_read_timeout(Some(std::time::Duration::from_secs(seconds)))
            .map_err(oops)
    }

    /// `n` bytes from a small generator.
    ///
    /// They mask a frame, which the protocol requires of a client and
    /// which stops a proxy misreading the stream. Nothing here is a
    /// secret: the connection is to a process this build just started, on
    /// the loopback address.
    fn bytes(&mut self, n: usize) -> Vec<u8> {
        (0..n)
            .map(|_| {
                // xorshift64*, which is short and good enough to mask.
                self.seed ^= self.seed >> 12;
                self.seed ^= self.seed << 25;
                self.seed ^= self.seed >> 27;
                (self.seed.wrapping_mul(0x2545F4914F6CDD1D) >> 32) as u8
            })
            .collect()
    }
}

/// Standard base64, for the handshake's key.
fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32;
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(ALPHABET[((n >> (18 - 6 * i)) & 0x3F) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The bytes a client puts on the wire for one text message.
    ///
    /// Written here rather than driven through a socket so the frame
    /// itself is checked: the header, the length encoding, and the
    /// masking a client is required to apply.
    fn framed(text: &str, mask: [u8; 4]) -> Vec<u8> {
        let payload = text.as_bytes();
        let mut frame = vec![0x81u8];
        match payload.len() {
            n if n < 126 => frame.push(0x80 | n as u8),
            n if n <= u16::MAX as usize => {
                frame.push(0x80 | 126);
                frame.extend_from_slice(&(n as u16).to_be_bytes());
            }
            n => {
                frame.push(0x80 | 127);
                frame.extend_from_slice(&(n as u64).to_be_bytes());
            }
        }
        frame.extend_from_slice(&mask);
        frame.extend(payload.iter().enumerate().map(|(i, b)| b ^ mask[i % 4]));
        frame
    }

    #[test]
    fn a_short_message_is_one_masked_text_frame() {
        let frame = framed("hi", [1, 2, 3, 4]);
        assert_eq!(frame[0], 0x81, "FIN and the text opcode");
        assert_eq!(frame[1], 0x82, "masked, and two bytes long");
        assert_eq!(&frame[2..6], &[1, 2, 3, 4]);
        assert_eq!(&frame[6..], &[b'h' ^ 1, b'i' ^ 2]);
    }

    #[test]
    fn a_longer_message_says_its_length_in_two_or_eight_bytes() {
        // 125 is the last length that fits in the header's own byte.
        assert_eq!(framed(&"x".repeat(125), [0; 4])[1], 0x80 | 125);
        let medium = framed(&"x".repeat(126), [0; 4]);
        assert_eq!(medium[1], 0x80 | 126);
        assert_eq!(u16::from_be_bytes([medium[2], medium[3]]), 126);
        // A CDP message is routinely over 64 kB — a page's own HTML comes
        // back this way — so the eight-byte form is not theoretical.
        let long = framed(&"x".repeat(70000), [0; 4]);
        assert_eq!(long[1], 0x80 | 127);
        assert_eq!(u64::from_be_bytes(long[2..10].try_into().unwrap()), 70000);
    }

    #[test]
    fn masking_is_reversible_which_is_the_whole_of_what_it_is_for() {
        let mask = [0x37, 0xFA, 0x21, 0x3D];
        let text = "{\"id\":1,\"method\":\"Page.navigate\"}";
        let frame = framed(text, mask);
        let body = &frame[6..];
        let back: Vec<u8> = body
            .iter()
            .enumerate()
            .map(|(i, b)| b ^ mask[i % 4])
            .collect();
        assert_eq!(String::from_utf8(back).unwrap(), text);
    }

    #[test]
    fn base64_is_the_one_everybody_elses_handshake_expects() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(base64(b"foob"), "Zm9vYg==");
        assert_eq!(base64(b"fooba"), "Zm9vYmE=");
        assert_eq!(base64(b"foobar"), "Zm9vYmFy");
        // The handshake's key is sixteen bytes, which is 24 characters.
        assert_eq!(base64(&[0u8; 16]).len(), 24);
    }
}
