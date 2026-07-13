use super::constants::SYSEX_REALTIME;

pub fn decode_14bit(lo: u8, hi: u8) -> u16 {
    u16::from(lo) | (u16::from(hi) << 7)
}

pub fn encode_14bit(value: u16) -> [u8; 2] {
    [
        (value as u8) & SYSEX_REALTIME,
        ((value >> 7) as u8) & SYSEX_REALTIME,
    ]
}

pub fn append_14bit(buf: &mut Vec<u8>, value: u16) {
    let [lo, hi] = encode_14bit(value);
    buf.push(lo);
    buf.push(hi);
}

pub fn append_7bit_data(buf: &mut Vec<u8>, data: &[u8]) {
    for &byte in data {
        buf.push(byte & SYSEX_REALTIME);
        buf.push((byte >> 7) & SYSEX_REALTIME);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_14bit() {
        for value in [0u16, 1, 127, 128, 255, 1023, 16383] {
            let [lo, hi] = encode_14bit(value);
            assert_eq!(decode_14bit(lo, hi), value);
        }
    }
}
