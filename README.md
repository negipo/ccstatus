# ccstatus

Claude Code 用のステータスラインフォーマッタ。stdin から StatusJSON を受け取り、固定レイアウトのステータスラインを出力する。

```
12.5% | my-repo | main | (+42,-10)
```

レイアウト: Context % | Git Root Dir | Git Branch | Git Changes

## インストール

Rust ツールチェーンが必要。

```bash
cargo install --git https://github.com/negipo/ccstatus
```

ローカルからインストールする場合:

```bash
git clone https://github.com/negipo/ccstatus.git
cd ccstatus
cargo install --path .
```

`~/.cargo/bin` が PATH に含まれていることを確認する。

## Claude Code への設定

`~/.claude/settings.json` に以下を追加する:

```json
{
  "statusLine": {
    "type": "command",
    "command": "ccstatus",
    "padding": 0
  }
}
```
