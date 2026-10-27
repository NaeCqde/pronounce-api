# HTTP API

## `GET /pronounce`

The romaji of a Japanese text, as `text/plain; charset=utf-8`.

| Query | |
|---|---|
| `q` | The text. Required, 1 to `[convert] max_query_chars` characters (300 by default). |
| `loanwords` | `romaji` (default) or `english`: how to write a loanword that is spelled in hiragana. |

```
GET /pronounce?q=チートアイテム管理局のお仕事
cheat item kanri kyoku no oshigoto

GET /pronounce?q=ナカよしふらすとれーしょん！
naka yoshi furasutore-shon!

GET /pronounce?q=ナカよしふらすとれーしょん！&loanwords=english
naka yoshi frustration!
```

A bad request (`q` missing, empty or too long, an unknown `loanwords`) is a `422` with
`{"detail": "..."}`. A text that cannot be converted is a `500`.

The query string has to be percent-encoded as UTF-8 (`encodeURIComponent` in JavaScript). Sending it
through a shell on Windows often mangles Japanese: use a program that encodes it.

## `GET /health`

`ok`, once the dictionary is loaded. The server starts listening only after that, so a connection
that is refused during start-up means it is not ready yet.

## Who calls it

`okazuri_bot` (`src/pronounce.ts`) calls `/pronounce` to rewrite a search query into romaji when a
search is meant to find pages that spell a title in the Latin alphabet.

## Replacing the Python server

This server answers like the FastAPI server in `okazuri_bot/api` (cutlet + pykakasi), with these
differences:

- no `engine` parameter: the pykakasi engine is gone (it was only a fallback);
- the dictionary is UniDic 2.1.2 (lindera) instead of UniDic 2.3.0 (unidic-lite), so a few words are
  split or read differently. `docs/ALGORITHM.md` has how close the two are.
