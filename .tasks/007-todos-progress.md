---
status: done
type: AFK
blocked-by: [004]
---

# Todos progress

## What to build

Parse TodoWrite, TaskCreate, and TaskUpdate calls from transcript. Render task progress on the activity line.

Format:
```
tasks 3/7
```

- TodoWrite: replaces entire todo list
- TaskCreate: adds a new task
- TaskUpdate: updates status of existing task (pending/in_progress/completed)
- Show completed count / total count

## Acceptance criteria

- [x] TodoWrite parsed: replaces todo list with new items
- [x] TaskCreate parsed: appends task with subject/description and status
- [x] TaskUpdate parsed: updates existing task status by taskId
- [x] Status normalization: pending/not_started -> pending, in_progress/running -> in_progress, completed/complete/done -> completed
- [x] Rendered as `tasks N/M` where N=completed, M=total
- [x] Dim color normally, green when all tasks completed
- [x] Hidden when no todos exist
