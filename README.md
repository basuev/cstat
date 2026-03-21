# cstat

status line for claude code. parses claude's streaming json output and renders a compact status bar with session info, usage, active tools, agents, and task progress.

## install

### from source

```
cargo install --git https://github.com/basuev/cstat
```

### binary download

download the latest release from [GitHub Releases](https://github.com/basuev/cstat/releases/latest):

| platform | architecture | binary |
|----------|-------------|--------|
| macOS | arm64 | `cstat-darwin-arm64` |
| macOS | x86_64 | `cstat-darwin-amd64` |
| Linux | x86_64 | `cstat-linux-amd64` |
| Linux | arm64 | `cstat-linux-arm64` |

```sh
curl -L -o cstat https://github.com/basuev/cstat/releases/latest/download/cstat-darwin-arm64
chmod +x cstat
mv cstat /usr/local/bin/
```

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
