---
status: done
type: AFK
blocked-by: [004]
---

# Tools activity line

## What to build

Render the second (activity) line showing tool status from transcript data.

Format:
```
Edit auth.ts  Grep x3  Read x2
```

- Active (running) tool: bright color, show tool name + target (filename, pattern, etc.)
- Last 3 completed tools: dim color, grouped by name with count
- Line hidden entirely when no tools activity exists

## Acceptance criteria

- [x] Running tool shown in bright color with name and target
- [x] Target extracted: file_path for Read/Write/Edit, pattern for Glob/Grep, truncated command for Bash
- [x] Completed tools grouped by name, showing count (e.g. `Read x3`)
- [x] Maximum 3 completed tool groups shown
- [x] Line omitted when no running tools and no recent completed tools
- [x] Separator from config used between tool entries
