---
status: done
type: AFK
blocked-by: [005, 006, 007, 008, 009, 010]
---

# Graceful degradation + render tests

## What to build

Final integration pass: ensure every module returns Option, all combinations of present/missing data render correctly. Comprehensive render tests.

## Acceptance criteria

- [x] Every module (stdin, transcript, git, usage, config) returns Option-wrapped data
- [x] All combinations of missing data produce valid output (no panics, no garbled lines)
- [x] Exit code 0 in all cases including: empty stdin, missing transcript, no git, no credentials, invalid config
- [x] Errors logged to stderr only, never to stdout
- [x] Render tests cover: all data present, each block missing individually, all blocks missing, colors on/off, custom separator, various threshold combinations
- [x] Activity line hidden when no tools/agents/todos
- [x] Minimal output with completely empty stdin: `[cstat] no data` or similar
