# Pronounce API

English | [日本語](README.ja.md)

Turns Japanese text into the romaji a search engine is asked for. A small HTTP server: one executable, the
dictionary inside it, nothing else to install.

```
チートアイテム管理局のお仕事   ->  cheat item kanri kyoku no oshigoto
ナカよしふらすとれーしょん！   ->  naka yoshi furasutore-shon!
失恋リカバリー                 ->  shitsuren rikabari-
```

- Hepburn romaji, in lowercase, with spaces between the words.
- A katakana loanword is written as the English word (チート is `cheat`).
- A long vowel that was written as `ー` stays a hyphen (`rikabari-`).
- A loanword spelled in hiragana (ふらすとれーしょん) is one word, and can be turned into English too
  (`loanwords=english`: `frustration`).

It replaces the Python server (FastAPI + cutlet) of [okazuri_bot](../okazuri_bot): the same `GET /pronounce`.

## Run it

```
pronounce-api                       start the server on 127.0.0.1:8000
pronounce-api serve --port 9000     options: --config <path> --host <address> --port <port>
pronounce-api convert "text"...     print the romaji (no server); without text, every line of stdin
pronounce-api --help
```

```
curl "http://127.0.0.1:8000/pronounce?q=%E3%83%81%E3%83%BC%E3%83%88"        # cheat
```

The query is percent-encoded UTF-8. The endpoints are in [docs/API.md](docs/API.md).

## Settings

`config.toml` in the current directory (the template is `config.example.toml`; every key is optional):

- `[server]` `host` / `port`. The default `127.0.0.1` is this machine only: put a reverse proxy in front of it.
- `[convert]` `max_query_chars`.
- `[exceptions]` words to write out instead of reading them from the dictionary.

## Downloads

Every push builds the archives (the executable, the READMEs, the config template, the license):

| Archive | |
|---|---|
| `pronounce-api-Linux-x86_64-musl` | **static**, no glibc: runs on any Linux x86_64 server (a VPS, a container) |
| `pronounce-api-Linux-x86_64`, `-Linux-aarch64` | glibc |
| `pronounce-api-Windows-x86_64`, `-Windows-aarch64` | |
| `pronounce-api-macOS-aarch64` | |

The executable is about 190 MB: UniDic is linked into it. How to run it on a server is in
[docs/DEPLOY.md](docs/DEPLOY.md).

## Build

```
cargo build --release          # downloads UniDic on the first build, so it needs the network
cargo test
```

A release build copies the executable next to `Cargo.toml` (set `PRONOUNCE_API_NO_COPY=1` to turn that off).
For musl: `rustup target add x86_64-unknown-linux-musl`, install `musl-tools`
(`apt install musl-tools`), then `cargo build --release --target x86_64-unknown-linux-musl`.

## How it works

[docs/ALGORITHM.md](docs/ALGORITHM.md): the dictionary, the rules that were taken from cutlet, what was added,
and how close it is to the Python version.

## License

MIT
