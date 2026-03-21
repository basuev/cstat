---
status: done
type: AFK
blocked-by: [001]
---

# Config (TOML)

## What to build

Implement config.rs that loads optional TOML config from `~/.claude/plugins/cstat/config.toml`. All fields optional with sensible defaults.

```toml
separator = "  "
colors = true
path_levels = 1
context_warning = 70
context_critical = 85
```

Wire config into render so that separator, colors toggle, path_levels, and thresholds are respected.

## Acceptance criteria

- [x] Config loaded from `~/.claude/plugins/cstat/config.toml`
- [x] Missing file -> all defaults applied, no error
- [x] Malformed TOML -> all defaults applied, error to stderr
- [x] `separator` controls block separation (default: double space)
- [x] `colors = false` produces output with no ANSI escape codes
- [x] `path_levels` (1-3) controls how many directory levels shown in project name
- [x] `context_warning` and `context_critical` override color thresholds
- [x] Partial config works (e.g. only `colors = false`, rest defaults)
