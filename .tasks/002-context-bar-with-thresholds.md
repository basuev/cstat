---
status: done
type: AFK
blocked-by: [001]
---

# Context bar with color thresholds

## What to build

Parse context_window data from stdin (current_usage.input_tokens and context_window_size). Calculate context percentage. Render a progress bar with ANSI color coding based on thresholds:

- <70%: green
- 70-85%: yellow
- >85%: red

First line becomes:
```
[Opus] my-project  ctx 45%
```

## Acceptance criteria

- [x] Context percentage calculated correctly from input_tokens / context_window_size
- [x] ANSI color applied: green below 70%, yellow 70-85%, red above 85%
- [x] Percentage displayed as integer (no decimals)
- [x] When context_window data is missing, ctx block is omitted (graceful degradation)
- [x] Double space separator between blocks
