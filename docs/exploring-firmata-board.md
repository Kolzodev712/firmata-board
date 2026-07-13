# Exploring firmata-board

A self-guided map for understanding this repo on your own. Follow the **progression** in order the first time through. Use **tips & tricks** whenever you get stuck.

---

## The 30-second picture

This crate runs on your **PC**, not on the Arduino. It speaks the [Firmata protocol](https://github.com/firmata/protocol) over serial to firmware (**StandardFirmata**) on the board.

```
Your Rust app
    → PinHandle / ArduinoUno     (typed API)
    → Board<T>                   (handshake, state, send/read)
    → protocol encode/decode     (pure bytes, no I/O)
    → Transport                  (serial or mock)
    ─────────── USB serial ───────────
    → StandardFirmata on Arduino
```

Four layers, dependencies flow **downward only**. The protocol layer never imports serial or board code. That separation is the whole design.

---

## Progression

Work through these phases in order. Each one unlocks the next. Skip ahead only if a phase already makes sense to you.

### Phase 0 — Orient yourself (5 min)

**Goal:** Know where things live before reading any implementation.

```
firmata-board/
├── README.md                 Usage, quick start
├── Cargo.toml                Features (`serial`), examples list
├── justfile                  fmt / clippy / test / audit
├── docs/architecture.md      Authoritative design doc — keep this open
├── examples/                 Runnable programs (hardware optional for reading)
├── src/
│   ├── lib.rs                Public API re-exports — read early
│   ├── error.rs              All error variants
│   ├── protocol/             Bytes in/out (no I/O)
│   ├── transport/            Serial + mock
│   ├── board/                Connect, handshake, state
│   └── pin/                  Typed pin/I2C handles
└── .cursor/docs/             This guide
```

Run once (no Arduino needed):

```bash
just check
# or: cargo test --no-default-features
```

If tests pass, you have a working dev environment. Everything from here is reading code.

---

### Phase 1 — The story (20 min)

**Goal:** Understand *why* the crate exists and *what* problem each layer solves.

Read in this order:

1. [`README.md`](../../README.md) — one usage example, dev commands
2. [`docs/architecture.md`](../../docs/architecture.md) — host vs device, layer diagram, data-flow sequence charts

**Stop when you can explain:** "The host sends Firmata bytes; the firmware executes pin operations. This crate is only the host side."

**Do not** read `src/` yet. The architecture doc is your map for everything that follows.

---

### Phase 2 — The public front door (10 min)

**Goal:** Know what callers are supposed to use.

Open [`src/lib.rs`](../../src/lib.rs). Every `pub use` here is intentional public API:

| Export | Role |
|--------|------|
| `ArduinoUno`, `d()`, `a()` | Open a Uno over serial; pin number helpers |
| `Board`, `PinHandle`, `I2cHandle` | Generic board + typed pin access |
| `MockTransport`, `Transport` | Testing and custom I/O |
| `protocol::*` | Low-level messages (advanced / tests) |
| `Error`, `Result` | Error handling |

**Stop when you can answer:** "If I use this crate in my app, I probably only touch `ArduinoUno`, `d()`, `a()`, and `PinHandle` methods."

---

### Phase 3 — Top-down: trace `blink` (45 min)

**Goal:** Follow one real user action from example code to serial bytes.

Start at [`examples/blink.rs`](../../examples/blink.rs) — the smallest example.

Read files in this exact order:

```
1. examples/blink.rs
2. examples/support/common.rs          → ArduinoUno::open(port)
3. src/board/profiles/uno.rs           → SerialTransport + Board::connect
4. src/board/handshake.rs              → what happens on connect
5. src/pin/mod.rs                      → .output(), .write_digital()
6. src/board/mod.rs                    → set_pin_mode, digital_write, send
7. src/protocol/encode.rs              → bytes being built
8. src/protocol/constants.rs           → command byte meanings
9. src/transport/serial.rs             → actual write to port (if using serial feature)
```

**Outbound path in one line:**

`PinHandle::write_digital` → `Board::digital_write` → `encode::set_digital_pin_value` → `Board::send` → transport

**Stop when you can draw** that chain from memory without looking at files.

---

### Phase 4 — Bottom-up: the protocol layer (30 min)

**Goal:** Understand Firmata as bytes, independent of boards or serial.

Read `src/protocol/` bottom to top:

```
1. constants.rs    → command bytes, pin modes
2. types.rs        → Message enum, PinMode, PinState
3. sysex.rs        → 7-bit SysEx encoding (used for I2C, firmware query, etc.)
4. encode.rs       → outgoing buffers + tests (golden bytes)
5. decode.rs       → Decoder::feed(byte) + tests
```

Run the protocol tests and read the assertions *before* the implementation:

```bash
cargo test encode --no-default-features -- --nocapture
cargo test decode --no-default-features -- --nocapture
```

Each test is a miniature spec: "given this intent, these exact bytes come out" or "given these bytes, this `Message` appears."

**Stop when you can answer:** "What does `Decoder::feed` return between messages? (`Ok(None)` until a full message is ready.)"

---

### Phase 5 — The glue: board layer (30 min)

**Goal:** Understand how connect, state, and retries tie protocol to transport.

Read `src/board/`:

```
1. state.rs        → BoardState::apply(message) — pin cache updates
2. handshake.rs    → perform_handshake — firmware/capability queries on connect
3. retry.rs        → retry_with_backoff — transient I/O only
4. mod.rs          → Board<T> — send, read_message, pin ops (re-read with new context)
```

Focus on the **inbound loop** in `Board::read_message`:

```
transport.read byte → Decoder::feed → Message → state.apply
```

Compare to Phase 3's outbound path. Together they are the full round trip.

Read [`src/board/handshake.rs`](../../src/board/handshake.rs) tests — they use `MockTransport` to simulate firmware replies without hardware.

**Stop when you can answer:** "Where does the cached pin value live?" (`BoardState.pins`, updated by `apply` and optimistically on writes.)

---

### Phase 6 — Transport boundary (15 min)

**Goal:** See where I/O is swappable.

Read `src/transport/`:

| File | Purpose |
|------|---------|
| `mod.rs` | `Transport` trait — anything `Read + Write` + flush |
| `mock.rs` | In-memory queue for tests |
| `serial.rs` | Real `serialport` (behind `serial` feature) |

Notice: `Board<T: Transport>` is generic over transport. Same board logic, real serial or mock.

**Stop when you can answer:** "How do I test board logic without a USB cable?" (`Board::connect(MockTransport::new())` or the handshake tests.)

---

### Phase 7 — Pin API and board profiles (20 min)

**Goal:** Understand how Uno-specific knowledge stays out of generic board code.

```
1. src/board/profiles/mod.rs   → BoardProfile trait
2. src/board/profiles/uno.rs   → pin counts, PWM pins, ArduinoUno wrapper
3. src/pin/mod.rs              → PinHandle, I2cHandle (re-read)
```

Key idea: `PinHandle<'a, T, P: BoardProfile>` uses the profile for analog pin mapping (`a(0)` → Firmata pin 14 on Uno) and validation.

**Stop when you can answer:** "What would I copy to add Arduino Mega?" (new file under `profiles/`, implement `BoardProfile`, add `ArduinoMega::open`.)

---

### Phase 8 — Examples catalog (pick your interest)

**Goal:** See how different Firmata features surface in the API.

| Example | What it teaches |
|---------|-----------------|
| `blink.rs` | Digital output — start here |
| `available.rs` | List serial ports, no board needed |
| `button.rs` | Digital input + reporting |
| `analog.rs` | Analog input + `enable_reporting()` |
| `pwm.rs` | PWM pin mode |
| `servo.rs` | Servo mode |
| `blinkm_i2c.rs` | I2C via `I2cHandle` |

All examples use [`examples/support/common.rs`](../../examples/support/common.rs). Port override:

```bash
export FIRMATA_PORT=/dev/ttyUSB0
cargo run --example blink --features serial
```

Read examples **after** Phases 3–7. They will click much faster.

---

### Phase 9 — Tests as living documentation (20 min)

**Goal:** Use tests to confirm and deepen understanding.

Find all tests:

```bash
rg '#\[test\]' src/
```

Priority reading order:

| File | Teaches |
|------|---------|
| `protocol/encode.rs` | Exact outgoing byte sequences |
| `protocol/decode.rs` | Parsing real Firmata traffic |
| `board/handshake.rs` | Full connect sequence with mocks |
| `board/state.rs` | State transitions from messages |
| `transport/mock.rs` | How mock I/O works |

**Stop when:** You can pick any test, read the input bytes, and point to the matching function in `encode.rs` or `decode.rs`.

---

### Phase 10 — You understand the whole thing when…

Check yourself. All of these should be yes:

- [ ] I can trace `uno.pin(d(13)).output()?.write_digital(true)` from example to serial bytes
- [ ] I can trace an incoming analog report from serial byte to `pin.value()`
- [ ] I know what happens inside `ArduinoUno::open` before the first pin operation
- [ ] I know why `protocol/` has zero imports from `transport/` or `board/`
- [ ] I know where errors are defined and which ones trigger retry
- [ ] I could add a new board profile without touching `encode.rs`
- [ ] I can run the full test suite without hardware

If any box is unchecked, go back to the phase that covers it.

---

## Tips & tricks

### How to read this codebase

**Start top-down, debug bottom-up.** Learn via examples and `PinHandle` first (Phase 3). When something breaks or surprises you, drop to `encode.rs` / `decode.rs` and the tests.

**Read tests before implementation.** Tests in `encode.rs` and `decode.rs` are shorter and more precise than the functions they exercise. Let the assertion tell you what to look for in the code.

**Read encode/decode as pairs.** Every outbound function in `encode.rs` has a corresponding inbound case in `decode.rs` and `types.rs`. When you learn `set_pin_mode`, immediately find how `Message::ReportPinMode` (or similar) comes back.

**Keep `architecture.md` open in a split pane.** It has the sequence diagrams. Refer back when you forget which layer owns what.

**One feature at a time.** Don't read I2C, servo, and analog reporting in one sitting. Follow one example (e.g. `blink` → `button` → `analog`) and master that slice of the API first.

### Command-line shortcuts

```bash
# Find definitions and usages
rg 'fn perform_handshake' src/
rg 'ArduinoUno' src/
rg 'set_digital_pin_value' src/

# List all tests
rg '#\[test\]' src/ -l

# Run one test module while reading that file
cargo test handshake --no-default-features -- --nocapture
cargo test encode:: --no-default-features

# Browse API docs in the browser
cargo doc --no-default-features --open

# See what the serial feature adds
cargo tree -e features

# See recent changes in an area you are reading
git log --oneline -- src/protocol/
git blame src/board/handshake.rs
```

### Understanding Firmata bytes

**Use `constants.rs` as a legend.** When you see a raw byte in a test, look it up there before googling.

**Keep the official spec nearby.** [Firmata protocol](https://github.com/firmata/protocol/blob/master/protocol.md) — correlate test bytes with spec sections. The crate targets Firmata 2.5.

**SysEx messages look weird on purpose.** Multi-byte values are split into 7-bit chunks (`sysex.rs`). When an I2C or firmware query looks opaque, start at `sysex.rs` then follow into `decode.rs`.

### Working without hardware

**All unit tests use `MockTransport`.** You never need an Arduino to understand or change protocol/board logic.

**`cargo test --no-default-features`** runs without the `serialport` dependency. This is the default CI path (`just check` uses it).

**Handshake tests are the best connect simulation.** They show exactly which SysEx queries run and which replies the board expects.

### Working with hardware

**Set the port explicitly.** Examples default to `/dev/ttyACM0`. Use `FIRMATA_PORT` or edit `examples/support/common.rs` temporarily.

**Enable tracing.** Examples call `tracing_subscriber::fmt::init()`. The crate instruments `send` and `read_message` with `tracing` — run with `RUST_LOG=debug` to see I/O without adding printlns:

```bash
RUST_LOG=firmata_board=debug cargo run --example blink --features serial
```

**Flash StandardFirmata first.** If `open` hangs or handshake fails, the firmware side is the usual culprit, not this crate.

### Navigating the code in an editor

**Start from `lib.rs` re-exports.** "Go to definition" on any public type lands you in the right module.

**Follow generic bounds.** `Board<T: Transport>` and `PinHandle<'a, T, P: BoardProfile>` — when confused about what methods exist, check the trait bounds and `impl` blocks, not guessed helpers.

**Feature-gated code.** `ArduinoUno` only exists with `feature = "serial"`. If go-to-definition fails, check `Cargo.toml` features or run `cargo doc --all-features --open`.

**Pin numbers: two namespaces.** `d(13)` is user-facing digital pin 13. Firmata also uses internal pin indices (analog pins offset by `ANALOG_PIN_OFFSET` = 14 on Uno). `BoardProfile::analog_to_firmata_pin` is the bridge — when analog looks wrong, check there first.

### When you are stuck

| Symptom | Likely layer | Look here |
|---------|--------------|-----------|
| Wrong bytes on the wire | protocol | `encode.rs` tests |
| Parse error / bad message | protocol | `decode.rs`, `types.rs` |
| Connect timeout | board | `handshake.rs` |
| Pin value stale or zero | board | `state.rs`, reporting enabled? |
| Port not found | transport / env | `serial.rs`, `FIRMATA_PORT` |
| Compile error on Uno type | features | enable `serial` feature |

**Git blame is for "why", not "what".** `git blame` on a line tells you when it changed; the test and architecture doc tell you what it does.

### Deliberate exercises

Do these to lock in understanding:

1. **Byte decode** — Pick a `decode.rs` test, write the expected `Message` variant on paper before running the test.
2. **Break the handshake** — In `handshake.rs` tests, change one mock reply byte and predict the error. Run to confirm.
3. **Add a log line** — Log the hex buffer in `Board::send`, run `blink` with hardware, match output to `encode.rs` tests.
4. **Follow a report** — In `button.rs` logic, trace how a digital report reaches `pin.value()`.
5. **Sketch the layers** — Draw the four-layer stack from memory. Compare to `architecture.md`.

---

## Quick layer reference

| Layer | Directory | Depends on | Knows about hardware? |
|-------|-----------|------------|------------------------|
| Pin API | `src/pin/` | board, profiles | Via profile only |
| Board | `src/board/` | protocol, transport | Via handshake/state |
| Protocol | `src/protocol/` | nothing external | No |
| Transport | `src/transport/` | std I/O | Port path only |

---

## Where to go next

| You want to… | Start here |
|--------------|------------|
| Use the crate in your project | `README.md`, `examples/` |
| Fix a protocol bug | `protocol/` tests first |
| Add Mega / Nano support | `board/profiles/`, `BoardProfile` trait |
| Improve reliability | `handshake.rs`, `retry.rs` |
| Add async serial | `architecture.md` roadmap; swap transport only |

---

## External references

- [Firmata protocol spec](https://github.com/firmata/protocol)
- [StandardFirmata firmware](https://github.com/firmata/arduino)
- [firmata-rs (upstream)](https://gitlab.com/Tiemen/firmata-rs) — see `NOTICE` in repo root
