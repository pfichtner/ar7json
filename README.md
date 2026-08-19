# ar7json

Standalone AR7 ↔ JSON converter for AVM FRITZ!Box `ar7.cfg` configuration files.

## What is AR7?

AR7 is the internal configuration language used by AVM FRITZ!Box routers. Configuration
files (`ar7.cfg`) contain nested blocks, key-value assignments, and values in various
formats (strings, integers, booleans, durations, identifiers, IP addresses, etc.).

This tool converts AR7 configuration files to a lossless JSON representation and back,
preserving all syntax distinctions and supporting arbitrary/unknown AVM configuration keys.

## Disclaimer

**This project does not claim to be an official AVM implementation or specification of
the AR7 configuration language.** It is a reconstructed parser based on analysis of
real-world configuration files. AR7 is a proprietary format owned by AVM GmbH.

## Installation

### From source

```bash
cargo install --path .
```

### Pre-built binaries

Download from the releases page for your platform.

## Building

```bash
cargo build --release
```

The binary is at `target/release/ar7json`.

## Usage

### Convert AR7 to JSON (lossless, canonical format)

```bash
ar7json to-json config.ar7
ar7json to-json config.ar7 -o config.json
```

### Convert JSON back to AR7

```bash
ar7json to-ar7 config.json
ar7json to-ar7 config.json -o config.ar7
```

### Check AR7 for syntax errors

```bash
ar7json check config.ar7
```

### Format AR7 with canonical formatting

```bash
ar7json format config.ar7
ar7json format config.ar7 -o formatted.ar7
```

### Pipe support (stdin/stdout)

```bash
cat config.ar7 | ar7json to-json > config.json
cat config.json | ar7json to-ar7 > config.ar7
cat config.ar7 | ar7json to-json | ar7json to-ar7 > config.roundtrip.ar7
```

### Simplified JSON mode (lossy, read-only)

```bash
ar7json to-json --simple config.ar7
```

## JSON Format

The lossless JSON format preserves all AR7 syntax distinctions:

```json
{
  "format": "ar7json",
  "version": 1,
  "document": {
    "entries": [
      {
        "key": "ar7cfg",
        "value": {
          "type": "object",
          "entries": [
            {
              "key": "mode",
              "value": {
                "type": "identifier",
                "value": "dsldmode_router"
              }
            },
            {
              "key": "igddenabled",
              "value": {
                "type": "boolean",
                "value": false,
                "raw": "no"
              }
            },
            {
              "key": "timeout",
              "value": {
                "type": "duration",
                "value": 1,
                "unit": "m",
                "raw": "1m"
              }
            }
          ]
        }
      }
    ]
  }
}
```

Value types: `string`, `integer`, `number`, `boolean`, `identifier`, `duration`,
`ip_address`, `mac_address`, `list`, `object`, `raw`.

## Round-trip Guarantees

```
AR7 → AST → JSON → AST → AR7' → AST' = AST
```

The parser produces deterministic output. Whitespace differences in serialization are
acceptable; syntactic/semantic AST differences are not.

## Limitations

- Comments are not preserved in serialized output (canonical formatting)
- The simplified JSON mode (`--simple`) does not support round-trip conversion
- Floating-point values with more than 4 dot-separated groups are not recognized as IP
  addresses (by design: the parser is conservative)

## Security Considerations

- AR7 files are treated as untrusted input
- The parser never executes, evaluates, or interprets configuration values
- No shell expansion, variable resolution, or network access
- Credential values in configurations are not logged by default

## License

MIT
