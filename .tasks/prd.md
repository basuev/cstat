# cstat - Minimal High-Performance Statusline for Claude Code

## Problem Statement

Claude Code не дает real-time видимость в состояние сессии. Пользователь не видит сколько контекста потрачено, какие инструменты работают прямо сейчас, насколько близок к rate limit, и сколько длится сессия. Существующее решение (claude-hud) написано на Node.js и имеет проблемы с перформансом: cold start ~30-50ms при каждом вызове (каждые ~300ms), полный перепарсинг транскрипта на каждый вызов, over-engineered конфигурация с 30+ параметрами.

## Solution

Нативный Rust бинарь cstat, который Claude Code вызывает как statusline subprocess каждые ~300ms. Показывает две строки: первая - модель, проект, контекст, usage лимиты (5h и 7d), длительность сессии. Вторая (опциональная) - текущая активность: tools, agents, todos. Упор на минимализм, производительность и практичность.

Ключевые перформанс-решения:
- Cold start <1ms (нативный бинарь, без runtime)
- Incremental парсинг транскрипта через mmap + offset (O(delta) вместо O(n))
- bincode для state между вызовами
- Ручной парсинг .git/HEAD вместо subprocess
- Синхронный HTTP (ureq) для usage API с кэшированием

## User Stories

1. As a Claude Code user, I want to see which model is currently active, so that I know what capabilities are available.
2. As a Claude Code user, I want to see current project name, so that I know which project I'm working in.
3. As a Claude Code user, I want to see context usage as a percentage with a progress bar, so that I know how much context is left.
4. As a Claude Code user, I want the context indicator to change color (green -> yellow -> red) based on thresholds, so that I get visual warnings before running out.
5. As a Claude Code user, I want to see 5-hour usage rate limit percentage with time remaining, so that I know when I'll hit the limit.
6. As a Claude Code user, I want to see 7-day usage rate limit percentage, so that I can pace my usage over the week.
7. As a Claude Code user, I want to see session duration, so that I know how long I've been working.
8. As a Claude Code user, I want to see which tool is currently running and its target file, so that I know what Claude is doing right now.
9. As a Claude Code user, I want to see recently completed tools with counts, so that I have context on what just happened.
10. As a Claude Code user, I want to see running subagents with their type, model, and duration, so that I know what's happening in parallel.
11. As a Claude Code user, I want to see todo/task progress as completed/total, so that I can track overall progress.
12. As a Claude Code user, I want the activity line to appear only when there's activity, so that the HUD stays minimal when idle.
13. As a Claude Code user, I want to see git branch name, so that I know which branch I'm on.
14. As a Claude Code user, I want to see a dirty indicator when there are uncommitted changes, so that I don't forget to commit.
15. As a Claude Code user, I want to configure the separator between blocks, so that I can use pipes or spaces based on preference.
16. As a Claude Code user, I want to disable colors, so that it works in terminals without color support.
17. As a Claude Code user, I want to configure how many path levels are shown, so that I see the right amount of project path context.
18. As a Claude Code user, I want to configure context warning/critical thresholds, so that I can tune when color changes happen.
19. As a Claude Code user, I want cstat to work without any config file, so that it works out of the box with sensible defaults.
20. As a Claude Code user, I want cstat to gracefully degrade when data is unavailable, so that it never shows errors in the statusline.
21. As a Claude Code user, I want to install cstat via cargo install, so that setup is one command.
22. As a Claude Code user, I want prebuilt binaries for my platform, so that I don't need Rust toolchain.
23. As a Claude Code user, I want cstat to have <1ms cold start, so that it doesn't add latency to Claude Code's UI loop.
24. As a Claude Code user, I want cstat to incrementally parse the transcript, so that performance doesn't degrade on long sessions.
25. As a Claude Code user, I want usage API results to be cached, so that cstat doesn't spam the API every 300ms.

## Implementation Decisions

### Language and Runtime
- Rust, no async runtime, synchronous execution
- Target: single static binary per platform

### Data Flow
1. Claude Code invokes cstat every ~300ms as subprocess
2. cstat reads JSON from stdin (model, context window, transcript_path, cwd)
3. Loads bincode state from `/tmp/cstat-<transcript_hash>.bin`
4. mmap transcript file, seek to saved offset, parse only new JSONL lines
5. Optionally fetch usage from Anthropic API (cached, ureq, TTL 60s success / 15s failure)
6. Read .git/HEAD for branch, check .git/index mtime for dirty
7. Render two lines with ANSI colors to stdout
8. Save updated state to bincode file

### Layout
Line 1 (always shown):
```
[Opus] my-project  ctx 45%  5h 25% (1h30m)  7d 60%  12m
```
- Model name in brackets
- Project name (configurable 1-3 path levels)
- Context percentage with color thresholds
- 5-hour usage percentage with time remaining
- 7-day usage percentage
- Session duration
- Blocks separated by configurable separator (default: double space)

Line 2 (shown only when activity exists):
```
Edit auth.ts  Grep x3  Read x2  explore[haiku] 2m15s  tasks 3/7
```
- Active tool: bright color, with target filename
- Last 3 completed tools: dim, grouped by name with count
- Running agents: yellow, with type, model, duration
- Task progress: dim (green when all complete)

### Visual Design
- Color + configurable separator (default double space) for block separation
- No pipe `|` separators by default
- Context thresholds: <70% green, 70-85% yellow, >85% red
- Colors can be disabled globally via config

### Transcript Parsing (Incremental)
- mmap the transcript JSONL file via memmap2
- Store byte offset of last read position in state
- On each invocation, seek to offset, parse only new lines
- Track inode + file size to detect new session (reset state)
- Parse tool_use/tool_result blocks for tools
- Parse Task tool calls for agents
- Parse TodoWrite/TaskCreate/TaskUpdate for todos
- Extract session_start from first timestamp

### State Management
- bincode serialized state in `/tmp/cstat-<hash>.bin`
- Hash derived from transcript_path
- Contains: byte offset, inode, tools map, agents map, todos, session_start, git index mtime, usage cache
- If transcript_path absent: stateless mode, no state file

### Git Integration
- Read `.git/HEAD` directly for branch name (parse `ref: refs/heads/<branch>`)
- Track `.git/index` mtime in state for dirty detection
- No subprocess, no libgit2 dependency
- Graceful fallback if not in git repo

### Usage API
- HTTP GET to `api.anthropic.com/api/oauth/usage` via ureq (synchronous)
- OAuth token from `~/.claude/.credentials.json`
- Cache successful response for 60s, failed for 15s, stored in state
- Show both 5-hour and 7-day limits
- Skip entirely if credentials not found (graceful degradation)

### Configuration
File: `~/.claude/plugins/cstat/config.toml` (optional)
```toml
separator = "  "
colors = true
path_levels = 1
context_warning = 70
context_critical = 85
```
All fields optional, defaults used when absent or file missing.

### Error Handling
- Every module returns Option - None means "skip this block"
- Exit code 0 always
- Errors logged to stderr only
- Never panic in production path

### Dependencies (6 crates)
- serde + serde_json: JSON parsing
- bincode: state serialization
- toml: config parsing
- ureq: HTTP for usage API
- memmap2: memory-mapped transcript

### Distribution
- `cargo install cstat`
- Prebuilt binaries on GitHub Releases for: macOS arm64, macOS x86_64, Linux x86_64, Linux arm64
- User manually adds statusLine command to `~/.claude/settings.json`

### Project Structure
```
src/
  main.rs          - entry point, orchestration
  stdin.rs         - parse Claude Code JSON input
  transcript.rs    - mmap + incremental JSONL parsing
  state.rs         - bincode state load/save
  usage.rs         - Anthropic Usage API client
  git.rs           - .git/HEAD + dirty detection
  config.rs        - TOML config with defaults
  render.rs        - ANSI line formatting
  types.rs         - shared data structures
```

## Testing Decisions

Good tests for cstat:
- Test external behavior through module public interfaces, not internal implementation
- Use realistic fixture data (actual Claude Code JSON/JSONL samples)
- Test degradation paths (missing data, malformed input, missing files)

### Modules with tests

**transcript** (deep module, most complex logic):
- Parses tool_use/tool_result pairs correctly
- Tracks agents from Task tool calls
- Tracks todos from TodoWrite/TaskCreate/TaskUpdate
- Incremental parsing: only processes lines after offset
- Handles malformed JSONL lines gracefully
- Detects new session via inode/size change

**render** (deep module, visual output):
- Correct output for all combinations of available/missing data
- Color thresholds apply correctly
- Separator configuration works
- Colors disabled mode produces no ANSI escapes
- Activity line omitted when no activity
- Completed tools grouped and counted correctly

**usage** (deep module, external API):
- Parses API response correctly (5h and 7d data)
- Cache TTL honored (60s success, 15s failure)
- Graceful handling of missing credentials
- Graceful handling of network errors / non-200 responses

## Out of Scope

- GUI or TUI interface (this is strictly a statusline)
- Claude Code plugin system integration (distributed as standalone binary)
- Configurable colors per element (single global colors toggle)
- Compact/expanded layout modes (single fixed layout)
- Config counts (MCP, hooks, CLAUDE.md)
- Ahead/behind remote tracking for git
- Windows native support (use WSL)
- Interactive configuration wizard
- Auto-update mechanism

## Further Notes

- The statusline API contract with Claude Code: stdin receives JSON with model, context_window, transcript_path, cwd. stdout lines are displayed as-is by Claude Code. Process invoked every ~300ms.
- bincode format is not stable across versions - if state struct changes, old state files should be silently discarded (version field in state).
- The `/tmp` state file approach means state is lost on reboot, which is acceptable since Claude Code sessions don't survive reboots either.
- For debugging: `cstat 2>/tmp/cstat.log` captures stderr diagnostics.
