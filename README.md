# ccstatus

A status line formatter for Claude Code. Reads StatusJSON from stdin and outputs a fixed-layout status line.

```
ctx:49% 5h(~15:30):58% 7d(~04/28 03:00):68% | ~/src/github.com/negipo/ccstatus main:MS? | prod/ap-northeast-1
 |      |              |                     |                                |    |       |
 |      |              |                     |                                |    status  AWS Profile/Region
 |      |              |                     |                                branch
 |      |              |                     git root dir (~ abbreviated)
 |      |              7-day rate limit usage with reset time
 |      5-hour rate limit usage with reset time
 context window usage
```

Reset time is only displayed when usage exceeds 50%. The 7-day reset uses `mm/dd HH:MM` format; the 5-hour reset uses `HH:MM`.

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
