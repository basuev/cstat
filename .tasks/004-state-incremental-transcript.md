---
status: done
type: AFK
blocked-by: [001]
---

# State + incremental transcript parsing

## What to build

Implement state.rs (bincode load/save to `/tmp/cstat-<hash>.bin`) and transcript.rs (mmap + incremental JSONL parsing).

State contains: byte offset, inode, file size, tools map, agents map, todos list, session_start, git index mtime, usage cache.

Transcript parsing:
- mmap the file via memmap2
- Seek to saved byte offset
- Parse only new JSONL lines after offset
- Extract tool_use/tool_result blocks into tools map
- Save new offset to state
- Detect new session via inode/size mismatch -> reset state

State file path: `/tmp/cstat-<hash>.bin` where hash is derived from transcript_path. Include a version field in state struct to silently discard incompatible old state files.

## Acceptance criteria

- [x] State saved and loaded correctly via bincode
- [x] State file path derived from transcript_path hash
- [x] Incompatible state version silently discarded (fresh start)
- [x] mmap used for transcript file access
- [x] Only lines after saved offset are parsed (incremental)
- [x] tool_use blocks create running tool entries
- [x] tool_result blocks mark tools as completed/error
- [x] New session detected when inode or file size shrinks -> state reset
- [x] Missing transcript_path -> stateless mode, no state file
- [x] Tests: incremental parsing, tool tracking, session detection, malformed lines
