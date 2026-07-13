# Changelog

## [0.1.0] - 2026-07-13

Initial release of **firmata-board** — a multi-board Firmata host adapter.

- Layered architecture: `protocol/`, `transport/`, `board/`, `pin/`
- `ArduinoUno::open()` with typed `PinHandle` API (`d()`, `a()` helpers)
- Firmata 2.5 protocol encode/decode with incremental decoder
- Robust handshake, mock transport, golden-byte tests
- Optional `serial` feature (default on) via `serialport`

Derived from [firmata-rs](https://gitlab.com/Tiemen/firmata-rs) by Tiemen Schuijbroek.
