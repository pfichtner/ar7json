# ar7json

[![Release][release-badge]][release-link]
[![Build][ci-badge]][ci-link]
[![License: MIT][license-badge]][license-link]

[release-badge]: https://img.shields.io/github/v/release/pfichtner/ar7json
[release-link]: https://github.com/pfichtner/ar7json/releases/latest
[ci-badge]: https://github.com/pfichtner/ar7json/actions/workflows/build.yml/badge.svg
[ci-link]: https://github.com/pfichtner/ar7json/actions/workflows/build.yml
[license-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[license-link]: LICENSE

Standalone AR7 ↔ JSON converter for AVM FRITZ!Box `ar7.cfg` configuration files.

## What is AR7?

AR7 is the internal configuration language used by AVM FRITZ!Box routers. Configuration
files (`ar7.cfg`) contain nested blocks, key-value assignments, and values in various
formats (strings, integers, booleans, durations, identifiers, IP addresses, etc.).

This tool converts AR7 configuration files to a lossless JSON representation and back,
preserving value types and supporting arbitrary/unknown AVM configuration keys.
Comments and original formatting are preserved by the `format` command, but not
carried through the JSON representation (see [Limitations](#limitations)).

## Motivation

Comparing and manipulating AR7 configuration files directly is cumbersome, as there are
few tools that understand the format. JSON, on the other hand, has a mature ecosystem of
tools for normalization, diffing, merging, querying, and transformation. By converting AR7
to JSON, the full power of these existing tools can be applied to FRITZ!Box configurations,
and the results can be converted back to AR7 without loss. This avoids writing a new
dedicated tool for every common operation.

## Disclaimer

**This project does not claim to be an official AVM implementation or specification of
the AR7 configuration language.** It is a reconstructed parser based on analysis of
real-world configuration files. AR7 is a proprietary format owned by FRITZ! GmbH (formerly AVM GmbH).

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

Produces a plain nested-object representation (`{"ar7cfg": {"mode": "..."}}`). This is a
one-way export: type tags and raw source text are discarded, so `to-ar7` rejects this
output. Use it for inspection, diffing, or querying with tools like `jq`.

## JSON Format

The lossless JSON format preserves all AR7 value types and their exact source text
(`raw`), so AR7 can be reconstructed without loss:

```json
{
  "document": {
    "entries": [
      {
        "key": "ar7cfg",
        "value": {
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
                "raw": "no",
                "type": "boolean",
                "value": false
              }
            },
            {
              "key": "timeout",
              "value": {
                "raw": "1m",
                "type": "duration",
                "unit": "m",
                "value": 1
              }
            }
          ],
          "type": "object"
        }
      }
    ]
  },
  "format": "ar7json",
  "version": 1
}
```

Value types: `string`, `integer`, `number`, `boolean`, `identifier`, `duration`,
`ip_address`, `mac_address`, `list`, `object`, `raw`.

Comments and whitespace are **not** represented in the JSON output: `to-json` drops
them. Use the `format` command instead if you need to preserve comments.

## Round-trip Guarantees

```
AR7 → AST → JSON → AST → AR7' → AST' = AST   (structure and values)
```

Structure and values survive the JSON round-trip exactly; comments and whitespace
trivia do not (they are dropped by `to-json`). The `format` command operates directly
on the AST and preserves comments:

```
AR7 → format → AR7'   (comments preserved)
```

The parser produces deterministic output. Whitespace differences in serialization are
acceptable; syntactic/semantic differences other than lost trivia are not.

## Limitations

- Comments and whitespace are lost when converting through JSON (`to-json` →
  `to-ar7`); use `format` to preserve them
- Comments within list values (between comma-separated items) are rejected as syntax
  errors by the parser
- The simplified JSON mode (`--simple`) does not support round-trip conversion
- Floating-point values with more than 4 dot-separated groups are not recognized as IP
  addresses (by design: the parser is conservative)

## Security Considerations

- AR7 files are treated as untrusted input
- The parser never executes, evaluates, or interprets configuration values
- No shell expansion, variable resolution, or network access
- Credential values in configurations are not logged by default

## Development

The CLI has byte-exact golden tests powered by [trycmd](https://docs.rs/trycmd) in
`tests/cmd/*.trycmd` and `tests/cmd/*.toml`. They pin the exact stdout/stderr of
`to-json`, `format`, `to-json --simple`, `check`, and stdin piping for every fixture.

```bash
cargo test --test cli                          # run golden tests
TRYCMD=overwrite cargo test --test cli         # update goldens after an intentional change
TRYCMD=dump cargo test --test cli              # write actual output to dump/ for inspection
```

Review every regenerated golden diff before committing it. Note that trycmd normalizes
path separators, so backslashes in expected output are rendered as `/` inside the
golden files; this is a display/normalization convention, not the program's output.

## License

MIT
