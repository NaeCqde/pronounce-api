# Running it on a server

For a Linux x86_64 VPS or container use the **musl** archive (`pronounce-api-Linux-x86_64-musl`): the
executable is static, so it needs no glibc and no other file. It is about 190 MB because UniDic is inside, but
the dictionary is mapped from the file and only the parts that are used are read: measured on Windows, it
starts in 0.3 s and uses about 8 MB of memory after 300 requests (the operating system may keep the whole
file in its cache).

## systemd

```
sudo install -m 755 pronounce-api /usr/local/bin/pronounce-api
sudo useradd --system --no-create-home pronounce
```

`/etc/systemd/system/pronounce-api.service`:

```ini
[Unit]
Description=Pronounce API
After=network.target

[Service]
User=pronounce
ExecStart=/usr/local/bin/pronounce-api serve --host 127.0.0.1 --port 8000
Restart=on-failure
# the log level: error, warn, info (default), debug
Environment=RUST_LOG=info
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
PrivateTmp=true

[Install]
WantedBy=multi-user.target
```

```
sudo systemctl daemon-reload
sudo systemctl enable --now pronounce-api
curl http://127.0.0.1:8000/health
```

The server listens only after the dictionary is loaded, so right after a start the port is not open yet.
`systemctl stop` sends SIGTERM, which ends it cleanly.

An optional `config.toml` is read from the working directory (give the service a `WorkingDirectory=`), or
named with `--config`.

## In front of it

The default address `127.0.0.1` is on purpose: the server has no authentication and no rate limit. To open it up
put a reverse proxy in front, and limit what it does there. nginx:

```nginx
limit_req_zone $binary_remote_addr zone=pronounce:10m rate=10r/s;

server {
    listen 443 ssl;
    server_name pronounce.example.com;
    # ssl_certificate ...

    location = /pronounce {
        limit_req zone=pronounce burst=20 nodelay;
        proxy_pass http://127.0.0.1:8000;
    }
    location = /health {
        proxy_pass http://127.0.0.1:8000;
    }
    location / { return 404; }
}
```

## Docker

The static executable runs in an empty image:

```dockerfile
FROM scratch
COPY pronounce-api /pronounce-api
EXPOSE 8000
ENTRYPOINT ["/pronounce-api", "serve", "--host", "0.0.0.0"]
```

`--host 0.0.0.0` is needed in a container; publish the port to `127.0.0.1` only unless something in front of it
protects it (`-p 127.0.0.1:8000:8000`).
