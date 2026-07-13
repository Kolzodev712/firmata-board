# firmata-board

Multi-board [Firmata](https://github.com/firmata/protocol) host adapter for Arduino and compatible boards, written in Rust.

Layered architecture with board profiles (Arduino Uno first), typed pin API, and `serialport` transport.

## Quick start

1. Flash **StandardFirmata** onto your Arduino from the Arduino IDE.
2. Add the dependency:

```toml
[dependencies]
firmata-board = { git = "https://github.com/Kolzodev712/firmata-board" }
```

3. Connect and blink the built-in LED:

```rust
use firmata_board::{a, d, ArduinoUno};

let mut uno = ArduinoUno::open("/dev/ttyACM0")?;
uno.pin(d(13)).output()?.write_digital(true)?;
uno.pin(a(0)).analog_input()?.enable_reporting()?;
```

Set `FIRMATA_PORT` to override the default `/dev/ttyACM0` in examples.

## Examples

```bash
cargo run --example blink --features serial
cargo run --example analog --features serial
cargo run --example available --features serial
```

## Architecture

See **[docs/architecture.md](docs/architecture.md)** for the full design: protocol layer, transport, board profiles, and typed pin API. Also see **[docs/exploring-firmata-board.md](docs/exploring-firmata-board.md)** for a guided exploration and examples that demonstrate the crate in practice.

## Development

```bash
just check
cargo test --no-default-features
```

## Attribution

This project is derived from [firmata-rs](https://gitlab.com/Tiemen/firmata-rs) by Tiemen Schuijbroek, with substantial rework: layered modules, board profiles, typed pin API, and multi-board direction.

## License

Licensed under either of Apache-2.0 or MIT at your option. See `LICENSE-MIT`, `LICENSE-APACHE`, and `NOTICE`.
