---
status: done
type: AFK
blocked-by: []
---

# Scaffold + stdin + minimal render

## What to build

Bootstrap the Rust project from scratch. Set up Cargo.toml with all 6 dependencies (serde, serde_json, bincode, toml, ureq, memmap2). Create the 9-file src/ structure with minimal stubs. Implement stdin JSON parsing (model, context_window, transcript_path, cwd) and a minimal render that outputs a single line:

```
[Opus] my-project
```

The binary must compile, read stdin, and produce valid statusline output. This is the tracer bullet that proves the full data flow works end-to-end with Claude Code's statusline API.

## Acceptance criteria

- [x] `cargo build --release` produces a working binary
- [x] Binary reads JSON from stdin with fields: model.display_name, context_window.current_usage.input_tokens, context_window.context_window_size, transcript_path, cwd
- [x] Outputs `[ModelName] project-name` to stdout
- [x] Project name extracted from cwd (last N path components based on future config, default 1)
- [x] Graceful output when stdin is empty or invalid
- [x] Exit code 0 always
- [x] All 9 src/ files exist with at least stub implementations
- [x] types.rs contains shared data structures for StdinData, TranscriptData, Config, State, etc.
