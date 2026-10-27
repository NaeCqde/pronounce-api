# Pronounce API

[English](README.md) | 日本語

日本語の文を、検索に使うローマ字にする小さな HTTP サーバーです。実行ファイル 1 つで動き、辞書も中に入っているので、
ほかに入れるものはありません。

```
チートアイテム管理局のお仕事   ->  cheat item kanri kyoku no oshigoto
ナカよしふらすとれーしょん！   ->  naka yoshi furasutore-shon!
失恋リカバリー                 ->  shitsuren rikabari-
```

- ヘボン式のローマ字で、小文字、語ごとに空白で区切ります。
- カタカナの外来語は英単語にします(`チート` は `cheat`)。
- もともと伸ばし棒「ー」だった長音は、ハイフンのままにします(`rikabari-`)。
- ひらがなで書かれた外来語(`ふらすとれーしょん`)は 1 語につなぎます。`loanwords=english` を付けると英単語にもできます
  (`frustration`)。

[okazuri_bot](../okazuri_bot) の Python 版(FastAPI + cutlet)の置き換えです。`GET /pronounce` は同じです。

## 動かす

```
pronounce-api                       127.0.0.1:8000 でサーバーを起動
pronounce-api serve --port 9000     オプション: --config <パス> --host <アドレス> --port <ポート>
pronounce-api convert "文"...       ローマ字を表示(サーバーなし)。文がなければ標準入力の各行
pronounce-api --help
```

```
curl "http://127.0.0.1:8000/pronounce?q=%E3%83%81%E3%83%BC%E3%83%88"        # cheat
```

クエリは UTF-8 をパーセントエンコードしたものです。エンドポイントは [docs/API.md](docs/API.md) にあります。

## 設定

カレントディレクトリの `config.toml`(雛形は `config.example.toml`。どのキーも省略できます)。

- `[server]` `host` / `port`。既定の `127.0.0.1` はこのマシンだけです。外に出すときはリバースプロキシを前に置きます。
- `[convert]` `max_query_chars`(クエリの最大文字数)。
- `[exceptions]` 辞書から読まずに、決めた綴りで書く語。

## ダウンロード

push のたびに、実行ファイル・README・設定の雛形・ライセンスを入れたアーカイブが作られます。

| アーカイブ | |
|---|---|
| `pronounce-api-Linux-x86_64-musl` | **静的リンク**で glibc に依存しません。どの Linux x86_64 サーバー(VPS、コンテナ)でも動きます |
| `pronounce-api-Linux-x86_64`、`-Linux-aarch64` | glibc 版 |
| `pronounce-api-Windows-x86_64`、`-Windows-aarch64` | |
| `pronounce-api-macOS-aarch64` | |

実行ファイルは約 190 MB です(UniDic を埋め込んでいるため)。サーバーでの動かし方は [docs/DEPLOY.md](docs/DEPLOY.md) にあります。

## ビルド

```
cargo build --release          # 初回は UniDic をダウンロードするのでネットワークが要ります
cargo test
```

リリースビルドは、実行ファイルを `Cargo.toml` の隣にコピーします(`PRONOUNCE_API_NO_COPY=1` で止められます)。
musl 向けは、`rustup target add x86_64-unknown-linux-musl` と `musl-tools`(`apt install musl-tools`)を入れて、
`cargo build --release --target x86_64-unknown-linux-musl` です。

## 仕組み

[docs/ALGORITHM.md](docs/ALGORITHM.md): 辞書、cutlet から移した規則、足した処理、Python 版との近さ。

## ライセンス

MIT
