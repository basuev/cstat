---
status: done
type: AFK
blocked-by: [001]
---

# Git branch + dirty

## What to build

Implement git.rs that reads branch name and dirty status without subprocess or libgit2.

- Branch: parse `.git/HEAD` for `ref: refs/heads/<branch>`
- Dirty: compare `.git/index` mtime against value stored in state

Display in first line:
```
[Opus] my-project git:(main*)
```

## Acceptance criteria

- [x] Branch name read from `.git/HEAD` by parsing `ref: refs/heads/` prefix
- [x] Detached HEAD handled (show short hash or "detached")
- [x] Dirty indicator `*` shown when `.git/index` mtime changed since last check
- [x] mtime stored in state for comparison across invocations
- [x] Not in git repo -> git info omitted entirely (graceful degradation)
- [x] No subprocess spawned, no libgit2
- [x] Works with standard .git directory layout
