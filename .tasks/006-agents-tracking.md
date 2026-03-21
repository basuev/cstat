---
status: done
type: AFK
blocked-by: [004]
---

# Agents tracking

## What to build

Parse Task tool calls from transcript for agent tracking. Render running agents on the activity line.

Format:
```
explore[haiku] 2m15s
```

- Agent type (subagent_type from input)
- Model in brackets if specified
- Running duration
- Only show running agents (completed agents not shown)

## Acceptance criteria

- [x] Task tool_use blocks parsed: extract subagent_type, model, description, start_time
- [x] Task tool_result blocks mark agent as completed
- [x] Running agents rendered with type, model (if present), and duration
- [x] Completed agents not shown in activity line
- [x] Duration formatted as Xm Xs or Xs for short durations
- [x] Agent entries shown in yellow color
