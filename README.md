# ccstatus

Claude Code 用のステータスラインフォーマッタ。stdin から StatusJSON を受け取り、固定レイアウトのステータスラインを出力する。

```
49% | ~/src/github.com/negipo/ccstatus (main) [MS?] | prod/ap-northeast-1
```

レイアウト:

- Context % -- 75% を超えると赤色で表示
- `|`
- Git Root Dir (~ 付きフルパス) (Git Branch) [Git Status]
  - Git Status: M(modified/yellow), S(staged/green), ?(untracked/red), D(deleted/yellow), ⇡(ahead), ⇣(behind)
  - branch は purple で表示
- `|`
- AWS Profile/Region (yellow) -- AWS_PROFILE, AWS_REGION 環境変数から取得。未設定時は省略

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
