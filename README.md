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

### as claude code status line

add to `~/.claude/settings.json`:

```json
{
  "statusLine": {
    "type": "command",
    "command": "cstat"
  }
}
```

claude code will invoke cstat as a subprocess, piping session data to stdin. the status line updates automatically every ~300ms.

### standalone

pipe claude code's json stream into cstat directly:

```sh
claude --output-format stream-json | cstat
```

### what it shows

line 1 (always):
```
[Opus] my-project  ctx 45%  5h 25% (1h30m)  7d 60%  12m
```

line 2 (when there's activity):
```
Edit auth.ts  Grep x3  Read x2  explore[haiku] 2m15s  tasks 3/7
```

includes: model name, project directory, context window usage, rate limits with cooldown, session duration, active tools, subagents, and task progress.

## configuration

create `~/.config/cstat/config.toml`:

```toml
[colors]
enabled = true

[format]
separator = " | "
```
