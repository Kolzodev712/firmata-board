use std::collections::VecDeque;
use std::io::{self, Cursor, Read, Write};

use super::Transport;

#[derive(Debug, Default)]
pub struct MockTransport {
    written: Vec<u8>,
    read_queue: VecDeque<u8>,
}

impl MockTransport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_read(&mut self, data: &[u8]) {
        self.read_queue.extend(data);
    }

    pub fn written(&self) -> &[u8] {
        &self.written
    }

    pub fn clear_written(&mut self) {
        self.written.clear();
    }
}

impl Read for MockTransport {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.read_queue.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "mock transport read queue empty",
            ));
        }
        let n = buf.len().min(self.read_queue.len());
        for slot in &mut buf[..n] {
            *slot = self.read_queue.pop_front().expect("checked non-empty");
        }
        Ok(n)
    }
}

impl Write for MockTransport {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.written.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Transport for MockTransport {
    fn flush_transport(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub fn read_exact_from_cursor(cursor: &mut Cursor<Vec<u8>>, buf: &mut [u8]) -> io::Result<()> {
    let mut offset = 0;
    while offset < buf.len() {
        match cursor.read(&mut buf[offset..]) {
            Ok(0) => {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "unexpected EOF",
                ));
            }
            Ok(n) => offset += n,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_and_reads() {
        let mut transport = MockTransport::new();
        transport.push_read(&[1, 2, 3]);
        transport.write_all(b"abc").unwrap();
        assert_eq!(transport.written(), b"abc");
        let mut buf = [0u8; 3];
        transport.read_exact(&mut buf).unwrap();
        assert_eq!(buf, [1, 2, 3]);
    }
}
