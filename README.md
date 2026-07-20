# ac_lib

A Rust library for reading live telemetry from **Assetto Corsa** over its UDP
remote telemetry protocol, with a future goal of bridging that data to
physical **HID devices** (e.g. wheels, button boxes, dashboards).

Assetto Corsa exposes a UDP socket that streams car/session telemetry to any
client that completes a handshake and subscribes to updates. This crate
implements the client side of that protocol: sending the handshake/subscribe
messages and parsing the binary frames the server sends back into typed Rust
structs.

## References

- [AC remote telemetry protocol doc](https://docs.google.com/document/d/1KfkZiIluXZ6mMhLWfDX1qAGbvhGRC3ZUzjVIt5FQpp4/pub)
- [Frame layout spreadsheet](https://docs.google.com/spreadsheets/d/1PhWgG1B7cv38OEummTZOOItrE-yYRBpMI2nV92BfDFU/pubhtml?gid=0&single=true)
- [rickwest/ac-remote-telemetry-client](https://github.com/rickwest/ac-remote-telemetry-client/blob/master/src/parsers/RTCarInfoParser.js) (JS reference implementation)

## Project layout

```
ac_lib/
├── Cargo.toml        # crate manifest (async runtime: tokio; error handling: anyhow; buffers: bytes)
├── src/
│   ├── lib.rs        # public Client API: connect, send handshake/subscribe, receive events
│   └── parser.rs     # wire format: Device/Operation enums, Event parsing, byte-level helpers
```

### `src/lib.rs`

Exposes `Client`, the entry point for consumers of the library:

- `Client::new(remote_addr, device)` — binds a local UDP socket and connects
  it to the AC server's telemetry address.
- `Client::send_message(operation)` — sends a `Handshake`, `SubscribeUpdate`,
  `SubscribeSpot`, or `Dismiss` request.
- `Client::recv_event()` — awaits the next UDP packet and parses it into an
  `Event`.

### `src/parser.rs`

Contains the wire protocol details:

- `Device` — identifies what kind of client is connecting (iPhone, iPad,
  Android phone/tablet — this mirrors the mobile-app values AC's protocol
  expects, not necessarily the device this library runs on).
- `Operation` — the request types a client can send (`Handshake`,
  `SubscribeUpdate`, `SubscribeSpot`, `Dismiss`).
- `Event` — the response types a client can receive, dispatched by payload
  size:
  - `HandshakeResponse` (408 bytes) — car/driver/track identification.
  - `CarInfo` (328 bytes) — full per-frame car telemetry (speed, pedals,
    RPM, per-wheel slip/load/suspension data, world position, etc).
  - `LapInfo` (212 bytes) — lap completion data (parsing not yet
    implemented — see checklist).
- Byte-parsing helpers for UTF-8/UTF-16LE strings, bools, and per-wheel
  `[f32; 4]` groups.

## Installation

This crate is not yet published to crates.io. Add it as a path or git
dependency in your `Cargo.toml`:

```toml
[dependencies]
ac_lib = { path = "../ac_lib" }
# or
ac_lib = { git = "https://github.com/<your-org>/AssettoReader" }
```

Requires Rust 2024 edition (Rust 1.85+).

### Build & test

```bash
cargo build
cargo test
```

### Usage

```rust
use ac_lib::Client;
use parser::{Device, Operation}; // re-export as needed from your integration

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::new("127.0.0.1:9996", Device::IPhone).await?;

    client.send_message(Operation::Handshake).await?;
    client.send_message(Operation::SubscribeUpdate).await?;

    loop {
        let event = client.recv_event().await?;
        println!("{event:?}");
    }
}
```

You'll need Assetto Corsa's UDP telemetry server enabled (`UDP_PLUGIN` in
the game's config, default port `9996`) and pointed at the machine running
this client.

## Feature checklist

- [x] UDP socket connect/bind to AC server
- [x] Handshake / subscribe / dismiss request messages
- [x] `HandshakeResponse` frame parsing
- [x] `CarInfo` frame parsing (speed, pedals, RPM, gear, per-wheel physics,
      world position)
- [ ] `LapInfo` frame parsing (currently returns default/empty values)
- [ ] Exponential backoff / reconnect handling on connection loss
- [ ] HID device interface — abstract trait for output devices (wheels,
      button boxes, dashboards) to consume parsed `Event`s
- [ ] Concrete HID device implementations (force feedback wheels, shift
      lights, etc.)
- [ ] Integration/unit tests around frame parsing
- [ ] Publish to crates.io

## Roadmap: HID support

The next major goal is to expose an interface that lets different HID
devices (wheels, in particular) consume telemetry events without each
integration needing to know about the UDP/parsing layer. The rough shape:

1. Define a trait (e.g. `TelemetrySink` or `HidDevice`) that receives parsed
   `Event`s and translates them into device-specific output (force
   feedback, LEDs, displays).
2. Keep `ac_lib`'s UDP/parsing core device-agnostic; HID implementations
   live either in this crate behind feature flags or in separate
   downstream crates that depend on `ac_lib`.
3. Start with one concrete wheel implementation to validate the trait
   shape before generalizing further.
