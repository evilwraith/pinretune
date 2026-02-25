# PSConfig

PSConfig provides a small, focused parser and typed model for PinSound
configuration files used by PinSound sound packs and firmware. The module's
goals are:

- Parse pipe-delimited PinSound configuration strings into a stable, typed
	representation (AST).
- Preserve unknown or unrecognised commands so files can be round-tripped
	without losing data.
- Provide convenient accessors for commonly-used values (checksum seed,
	pspack id, app version, sound commands, etc.) and a simple serializer to
	write the configuration back to the canonical pipe-delimited format.

### Why this crate exists

PinSound configuration files are compact text blobs where individual
statements are separated by the `|` delimiter. Historically these files have
been handled as unstructured strings or key/value maps which made it hard to
add first-class support for sound commands (for example `355 stop 105`) or to
reliably extend the parser. `psconfig` provides a typed `Statement` enum and
`PsConfig` container that keeps statement order and enables reliable
round-trips.

### Features

- Typed AST: `Statement` enum variants for well-known primitives such as
	`checksum`, `checksum_version`, `pspack_id`, `max`/`ducking` entries and
	sound commands like `<id> stop <target>` and `<id> preload`.
- Unknown statements are preserved in `Statement::Unknown(String)`.
- `PsConfig::to_config_string()` serializes the AST back to the
	pipe-delimited format used by PinSound firmware.
- Small, dependency-free parser for the grammar used in practice by
	PinSound files.

### Quick usage

Parse a configuration string:

```rust
use psconfig::parse;

let txt = "checksum_version 1|pspack_id b00bfaceb00bface|checksum JeanLucPicard|";
let cfg = parse::parse(txt).expect("failed to parse");
assert_eq!(cfg.checksum(), Some("JeanLucPicard"));
```

Iterate sound commands:

```rust
for (id, action) in cfg.sound_commands() {
		println!("sound {} -> {:?}", id, action);
}
```

Serialize back to a config string (round-trip):

```rust
let out = cfg.to_config_string();
// `out` now contains pipe-delimited statements ending with `|`
```

### Extending the parser

To add support for a new PinSound primitive or action:

1. Add a new variant to the `Statement` enum in `src/config.rs`.
2. Implement the `Display` formatting for the variant so `to_config_string`
	 produces the correct textual form.
3. Add a parsing branch in `src/parse.rs::parse_statement` to construct the
	 variant from the input tokens.

### Design notes

- The parser deliberately falls back to `Statement::Unknown` for malformed
	or unrecognised statements rather than failing the whole parse. This keeps
	the crate tolerant of slightly different formats and preserves data.
- The crate keeps the order of statements which is important for
	deterministic round-trips and for tools that expect statements in a
	particular order.

#### Example config

```
checksum_version 1|pspack_id b00bfaceb00bface|checksum JeanLucPicard|355 stop 105 |500 stop 374 |700 stop 374 |703 stop 374 |709 stop 374 |730 stop 374 |731 stop 374 |732 stop 374 |max sfx 3|max voice 2|ducking sfx 90|ducking voice 85|ducking jingle 5|record_enabled 1|0 preload |timestamp 1681718217|versionDetail 22.05.1|versionConfigFile 1|
```

### Testing

The crate contains unit tests in `src/parse.rs` and `src/config.rs` demonstrating
parsing, round-tripping and handling of unknown statements. Run them with

```bash
cargo test -p psconfig
```

### Notes

- This crate intentionally keeps parsing logic simple and dependency-free so
	it is easy to embed, inspect and extend. If you need more complex parsing
	(for example stricter validation or richer RTTTL parsing) those can be
	implemented as separate modules that translate to/from the typed AST.

