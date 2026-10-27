# Changes

What the latest change (`feat: rewrite the pronounce API in Rust`) contains. The file is rewritten from
scratch for each commit.

## What was done

Rewrote the Japanese to romaji server of `okazuri_bot` (Python: FastAPI + cutlet + pykakasi, in `api/`) in
Rust. This folder started as a copy of `avatar-uploader`, so it keeps that project's layout (the split into
`main` / `cli` / `config` / `utils`, English code comments, `docs/` for the reasons behind the
implementation, tests inside each module, the CI with one build and archive per platform) and drops what was
specific to it (the GUI, the VRChat API, the avatar database, the file watcher).

- **Server**: `GET /pronounce?q=...[&loanwords=romaji|english]` and `GET /health` (axum). A bad request is a 422
  with `{"detail": ...}`, like the FastAPI server answered. Stops cleanly on Ctrl+C and SIGTERM.
- **CLI**: `serve` (default), `convert` (the text on the command line, or every line of stdin).
- **Settings**: `config.toml` (`config.example.toml` is the template): the address, the longest query, the
  exceptions.
- **Conversion** ([docs/ALGORITHM.md](docs/ALGORITHM.md)): cutlet's algorithm ported to Rust on lindera + the
  embedded UniDic (nothing to install next to the executable), and the changes the Python server made on top of it:
  - katakana loanwords are English (`チート` -> `cheat`);
  - a loanword in hiragana that UniDic cuts into pieces is one word (`furasutore-shon`);
  - `ー` is a hyphen (`rikabari-`);
  - `loanwords=english` looks the hiragana loanwords up in katakana (`frustration`);
  - exceptions for the words whose foreign spelling in UniDic is wrong (ラーメン, メイド, ...), which can be several
    words of the dictionary (クリベロン) and can be added in `config.toml`.
- **Differences from the Python server**: UniDic 2.1.2 (lindera) instead of 2.3.0 (unidic-lite), so the reading
  of a word is worked out from `pron` / `lForm` (`Word::reading`), and the `engine` parameter is gone. Of the 9,788
  titles in the okazuri_bot catalogue, 91% come out exactly the same; the rest is mostly a different split or
  reading of the dictionary (`docs/ALGORITHM.md`). Fixed on the way: `うぃ` (was `i`), `ゝ` (left a kana in the
  romaji), the middle dot `・`.
- **CI**: the tests on Windows / macOS / Linux, and a build and an archive for every platform: Linux x86_64,
  **Linux x86_64 musl (new: a static executable)**, Linux aarch64, Windows x86_64 / aarch64, macOS aarch64. Each
  executable is run once on the runner (a smoke test) before it is archived.

## Checked

- `cargo test`: 41 unit tests and 3 tests that run the executable on `tests/fixtures/golden.tsv`.
- The musl build was linked on Windows (`rust-lld`): a 64-bit x86-64 ELF with no interpreter, so statically linked.
  It was not run (no Linux here); the smoke test of the CI does that.
