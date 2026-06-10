# ccstatus

A status line formatter for Claude Code. Reads StatusJSON from stdin and outputs a fixed-layout status line.

```
ctx:49% 5h(~1h23m):58% 7d(~2d5h30m):68% | ~/src/github.com/negipo/ccstatus main:MS? | prod/ap-northeast-1
 |      |              |                  |                                |    |     |
 |      |              |                  |                                |    |     AWS Profile/Region
 |      |              |                  |                                |    status
 |      |              |                  |                                branch
 |      |              |                  git root dir (~ abbreviated)
 |      |              7-day rate limit usage with remaining time until reset
 |      5-hour rate limit usage with remaining time until reset
 context window usage
```

The remaining time until reset is only displayed when usage exceeds 50%, formatted like `1h23m` or `2d5h30m` (leading units are omitted when zero).

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
