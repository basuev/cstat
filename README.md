# cstat

status line for claude code. parses claude's streaming json output and renders a compact status bar with session info, usage, active tools, agents, and task progress.

## install

### from crates.io

```
cargo install cstat
```

### from source

```
cargo install --git https://github.com/basuev/cstat
```

### binary download

grab a binary from [releases](https://github.com/basuev/cstat/releases/latest):

```sh
curl -L -o cstat https://github.com/basuev/cstat/releases/latest/download/cstat-darwin-arm64
chmod +x cstat
mv cstat /usr/local/bin/
```

available binaries: `cstat-darwin-arm64`, `cstat-darwin-amd64`, `cstat-linux-amd64`, `cstat-linux-arm64`.

## usage

pipe claude code's json stream into cstat:

```sh
claude --output-format stream-json | cstat
```

## configuration

create `~/.config/cstat/config.toml`:

```toml
[colors]
enabled = true

[format]
separator = " | "
```
