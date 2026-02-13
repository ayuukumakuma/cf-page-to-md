# page2md

Cloudflare Browser Rendering の Markdown endpoint を使って、Webページを Markdown に変換する CLI ツールです。
非公式ツールであり、Cloudflare とは提携・承認関係にありません。
This is an unofficial tool and is not affiliated with or endorsed by Cloudflare.

## 前提

以下の環境変数を設定してください。

- `CF_ACCOUNT_ID`: Cloudflare アカウント ID
- `CF_API_TOKEN`: Browser Rendering を利用できる API トークン

## 使い方

```bash
page2md <URL> [--out-dir <DIR>] [--filename <NAME>] [--overwrite]
```

### 標準出力モード

`--out-dir` を指定しない場合、Markdown をそのまま標準出力に表示します。

```bash
page2md https://example.com
```

### 保存モード

`--out-dir` を指定した場合、指定ディレクトリに Markdown ファイルを保存します。

```bash
page2md https://example.com --out-dir ./out
```

保存時のファイル名ルール:

- `--filename` 指定時: 指定名を使用（`.md` 未指定なら自動付与）
- `--filename` 未指定時: URL 由来のスラッグ名を使用

上書きルール:

- デフォルト: 同名ファイルがあるとエラー
- `--overwrite` 指定時のみ上書き

## 利用規約と公開時の注意

- 本ツールは Cloudflare Browser Rendering API を利用する非公式CLIです。
- ツールの作成・公開・配布可否は、Cloudflareの適用規約（Website Terms だけでなく、Self-Serve Subscription Agreement / Enterprise Subscription Agreement / 個別契約を含む）に従って判断してください。
- APIトークンの権限範囲、利用上限、許可されたユースケースは契約と公式ドキュメントに基づいて運用してください。
- 本リポジトリは法的助言を提供するものではありません。最終判断は利用者・公開者の責任で行ってください。

## 固定の API パラメータ

このCLIは常に以下の `rejectRequestPattern` を送信し、`.css` リクエストを除外します。

```json
{
  "rejectRequestPattern": ["/^.*\\.(css)/"]
}
```

## AI フレンドリーなヘルプ

通常ヘルプ:

```bash
page2md --help
```

機械可読ヘルプ(JSON):

```bash
page2md --help-json
```

`--help-json` は API を呼び出しません。

## Nix Flake

### 開発環境

```bash
nix develop
```

### ビルド

```bash
nix build
```

### チェック

```bash
nix flake check
```

`checks` では以下を実行します。

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test --all-targets`
