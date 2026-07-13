#[cfg(feature = "serial")]
use std::time::Duration;

use crate::error::Result;
use crate::transport::Transport;

#[cfg(feature = "serial")]
use serialport::{ClearBuffer, DataBits, FlowControl, Parity, StopBits};

#[cfg(feature = "serial")]
const ARDUINO_RESET_DELAY: Duration = Duration::from_millis(2000);

#[cfg(feature = "serial")]
pub struct SerialTransport {
    port: Box<dyn serialport::SerialPort>,
}

#[cfg(feature = "serial")]
impl SerialTransport {
    pub fn open(path: &str, baud: u32) -> Result<Self> {
        let port = serialport::new(path, baud)
            .data_bits(DataBits::Eight)
            .parity(Parity::None)
            .stop_bits(StopBits::One)
            .flow_control(FlowControl::None)
            .timeout(Duration::from_millis(1000))
            .open()
            .map_err(|source| crate::error::Error::StdIoError {
                source: source.into(),
            })?;

        // Opening the port toggles DTR and resets the board; wait for boot then drain.
        std::thread::sleep(ARDUINO_RESET_DELAY);
        port.clear(ClearBuffer::All)
            .map_err(|source| crate::error::Error::StdIoError {
                source: source.into(),
            })?;

        Ok(Self { port })
    }
}

#[cfg(feature = "serial")]
impl std::io::Read for SerialTransport {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.port.read(buf)
    }
}

#[cfg(feature = "serial")]
impl std::io::Write for SerialTransport {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.port.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.port.flush()
    }
}

#[cfg(feature = "serial")]
impl Transport for SerialTransport {
    fn flush_transport(&mut self) -> std::io::Result<()> {
        self.port.flush()
    }
}
