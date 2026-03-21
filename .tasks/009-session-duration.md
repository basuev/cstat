---
status: done
type: AFK
blocked-by: [004]
---

# Session duration

## What to build

Extract session start time from the first timestamp in the transcript. Calculate and display session duration in the first line.

Format:
```
[Opus] my-project  ctx 45%  12m
```

or for longer sessions: `1h 30m`

## Acceptance criteria

- [x] session_start extracted from first entry's timestamp in transcript
- [x] Duration = now - session_start
- [x] Format: `<1m` for under a minute, `Xm` for minutes, `Xh Xm` for hours
- [x] Displayed in dim color on first line
- [x] Omitted when transcript has no timestamps (graceful degradation)
