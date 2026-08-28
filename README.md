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

## Why Rust?

This project is written in Rust because the requirements map well to Rust's strengths:

- **Recursive AST with rich value types.** AR7 values (strings, durations, IP addresses,
  identifiers, etc.) are naturally modeled as a tagged enum — Rust's `enum` with data is
  the most concise and type-safe way to express this.
- **Standalone binary with no runtime dependencies.** Users download a single executable
  that runs on Linux, macOS, and Windows without an interpreter or VM. Cross-compilation
  to all six target platforms is straightforward.
- **Mature CLI ecosystem.** `clap`, `serde_json`, and `miette` provide best-in-class
  argument parsing, JSON serialization, and error reporting with minimal boilerplate.

Alternatives like Go or Python could work, but would require more verbose type modeling
(Go) or sacrifice single-binary distribution (Python).

## Use Cases

### Compare two router backups

Export the configuration from your router twice (e.g., before and after a change),
convert both to JSON, and diff them:

```bash
ar7json to-json old.ar7 -o old.json
ar7json to-json new.ar7 -o new.json
diff old.json new.json
```

Or use a structured diff tool like `delta` or `git diff` for readable output.

### Bulk-edit configuration with jq

Change a setting across the entire config without opening a GUI. For example, switch
all DNS servers:

```bash
cat config.ar7 \
  | ar7json to-json \
  | jq '.document.entries[0].value.entries += [{ key: "dnsserver1", value: { type: "ip_address", value: "1.1.1.1" } }]' \
  | ar7json to-ar7 -o config.ar7
```

### Version-control router configurations

Store your router config in Git as JSON for meaningful diffs and a clear commit
history. Restore by converting back:

```bash
ar7json to-json config.ar7 -o config.json
git add config.json && git commit -m "disable guest wifi"

# later, to restore:
ar7json to-ar7 config.json -o config.ar7
```

### Validate before deploying

Check for syntax errors in CI or deployment scripts before pushing a config to the
router:

```bash
if ar7json check config.ar7; then
  echo "Config is valid, deploying..."
else
  echo "Config has errors, aborting."
  exit 1
fi
```

### Inspect a config without the raw format

Use the simplified JSON mode to get a clean, queryable view of your router's settings
for documentation or exploration:

```bash
ar7json to-json --simple config.ar7 | jq .
```

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

Download from the releases page for your platform. Tarballs and packages include
symlinks for short command names (`ar7-to-json`, `json-to-ar7`, `ar7-check`, `ar7-fmt`).

## Building

```bash
cargo build --release
```

The binary is at `target/release/ar7json`.

## Usage

### Short commands (via symlinks)

Release tarballs and packages include symlinks for shorter command names. These are
symlinks to the main binary and accept the same flags:

| Subcommand | Symlink |
|------------|---------|
| `ar7json to-json` | `ar7-to-json` |
| `ar7json to-ar7` | `json-to-ar7` |
| `ar7json check` | `ar7-check` |
| `ar7json format` | `ar7-fmt` |

```bash
ar7-to-json config.ar7 -o config.json
json-to-ar7 config.json -o config.ar7
ar7-check config.ar7
ar7-fmt config.ar7 -o formatted.ar7
```

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

### Generate shell completions

```bash
ar7json completions bash > ~/.local/share/bash-completion/completions/ar7json
ar7json completions zsh > "${fpath[1]}/_ar7json"
ar7json completions fish > ~/.config/fish/completions/ar7json.fish
```

Supported shells: `bash`, `zsh`, `fish`, `powershell`, `elvish`. Use `-o` to write to a file.

### Generate man page

```bash
ar7json man > ~/.local/share/man/man1/ar7json.1
ar7json man -o ar7json.1
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
`to-json`, `format`, `to-json --simple`, `check`, stdin piping, `completions`, and
`man` for every fixture. The `completions`/`man` commands are additionally covered by
structural integration tests in `tests/generate.rs`.

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
