use std::io::{self, Read, Write};

pub trait Transport: Read + Write {
    fn flush_transport(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub struct FlushTransport<T> {
    inner: T,
}

impl<T> FlushTransport<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T: Read + Write> Read for FlushTransport<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

impl<T: Read + Write> Write for FlushTransport<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl<T: Read + Write> Transport for FlushTransport<T> {
    fn flush_transport(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

#[cfg(feature = "serial")]
pub mod serial;

pub mod mock;

pub use mock::MockTransport;
