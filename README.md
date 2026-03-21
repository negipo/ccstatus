# ccstatus

A status line formatter for Claude Code. Reads StatusJSON from stdin and outputs a fixed-layout status line.

```
ctx:49% 5h:30% 7d:68% | ~/src/github.com/negipo/ccstatus main:MS? | prod/ap-northeast-1
```

Layout:

- ctx: Context window usage -- displayed in red when exceeding 75%
- 5h: 5-hour window usage -- yellow above 50%, red above 75%
- 7d: 7-day window usage -- yellow above 50%, red above 75%
- `|`
- Git root dir (full path with ~) Branch:Status
- `|`
- AWS Profile/Region -- sourced from AWS_PROFILE and AWS_REGION environment variables; omitted when unset

## Installation

Requires the Rust toolchain.

```bash
cargo install --git https://github.com/negipo/ccstatus
```

To install from a local clone:

```bash
git clone https://github.com/negipo/ccstatus.git
cd ccstatus
cargo install --path .
```

Make sure `~/.cargo/bin` is in your PATH.

## Claude Code Configuration

Add the following to `~/.claude/settings.json`:

```json
{
  "statusLine": {
    "type": "command",
    "command": "ccstatus",
    "padding": 0
  }
}
```
