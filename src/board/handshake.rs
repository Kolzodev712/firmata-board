use std::time::{Duration, Instant};

use crate::board::Board;
use crate::error::{Error, Result};
use crate::protocol::{encode, Message};
use crate::transport::Transport;

pub const DEFAULT_HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(5);

pub fn perform_handshake<T: Transport>(board: &mut Board<T>, timeout: Duration) -> Result<()> {
    let deadline = Instant::now() + timeout;

    board.send(&encode::query_firmware())?;

    let mut got_firmware = false;
    let mut got_version = false;
    while !(got_firmware && got_version) {
        if Instant::now() > deadline {
            return Err(Error::HandshakeFailed {
                reason: "timed out waiting for firmware/version".into(),
            });
        }
        let msg = board.read_message()?;
        match &msg {
            Message::ReportFirmware { .. } => got_firmware = true,
            Message::ProtocolVersion { .. } => got_version = true,
            _ => {}
        }
    }

    board.send(&encode::query_capabilities())?;
    board.send(&encode::query_analog_mapping())?;

    let mut got_capabilities = false;
    let mut got_mapping = false;
    while !(got_capabilities && got_mapping) {
        if Instant::now() > deadline {
            return Err(Error::HandshakeFailed {
                reason: "timed out waiting for capabilities/mapping".into(),
            });
        }
        let msg = board.read_message()?;
        match &msg {
            Message::CapabilityResponse { .. } => got_capabilities = true,
            Message::AnalogMappingResponse { .. } => got_mapping = true,
            _ => {}
        }
    }

    let port_count = board.state().port_count();
    for port in 0..port_count {
        board.send(&encode::report_digital(port, true))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Board;
    use crate::protocol::constants::*;
    use crate::transport::mock::MockTransport;

    fn uno_capability_response() -> Vec<u8> {
        vec![
            START_SYSEX,
            CAPABILITY_RESPONSE,
            PIN_MODE_INPUT,
            1,
            PIN_MODE_OUTPUT,
            1,
            PIN_MODE_IGNORE,
            PIN_MODE_INPUT,
            1,
            PIN_MODE_OUTPUT,
            1,
            PIN_MODE_PWM,
            10,
            PIN_MODE_IGNORE,
            END_SYSEX,
        ]
    }

    fn uno_analog_mapping() -> Vec<u8> {
        vec![
            START_SYSEX,
            ANALOG_MAPPING_RESPONSE,
            127,
            127,
            127,
            127,
            127,
            127,
            127,
            127,
            127,
            127,
            127,
            127,
            127,
            127,
            2,
            2,
            2,
            2,
            2,
            2,
            END_SYSEX,
        ]
    }

    #[test]
    fn handshake_out_of_order_version_first() {
        let mut transport = MockTransport::new();
        transport.push_read(&[REPORT_VERSION, 2, 5]);
        transport.push_read(&[
            START_SYSEX,
            REPORT_FIRMWARE,
            2,
            5,
            b'S',
            b't',
            b'd',
            END_SYSEX,
        ]);
        transport.push_read(&uno_capability_response());
        transport.push_read(&uno_analog_mapping());

        let mut board = Board::from_transport(transport);
        perform_handshake(&mut board, DEFAULT_HANDSHAKE_TIMEOUT).unwrap();
        assert_eq!(board.state().protocol_version, "2.5");
        assert!(!board.state().firmware_name.is_empty());
        assert!(board.state().pin_count() > 1);
    }
}
