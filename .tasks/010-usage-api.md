---
status: done
type: AFK
blocked-by: [001, 003]
---

# Usage API (5h + 7d)

## What to build

Implement usage.rs that fetches rate limit data from Anthropic Usage API via ureq (synchronous HTTP).

- Read OAuth token from `~/.claude/.credentials.json` (claudeAiOauth.accessToken)
- GET `api.anthropic.com/api/oauth/usage`
- Parse 5-hour and 7-day usage percentages and reset times
- Cache successful response for 60s, failed for 15s (stored in state)

Display in first line:
```
[Opus] my-project  ctx 45%  5h 25% (1h30m)  7d 60%  12m
```

## Acceptance criteria

- [x] OAuth token read from `~/.claude/.credentials.json`
- [x] HTTP GET to usage API endpoint via ureq
- [x] 5-hour usage: percentage + time remaining until reset
- [x] 7-day usage: percentage
- [x] Cache TTL: 60s for success, 15s for failure
- [x] Cache stored in bincode state
- [x] Missing credentials -> usage blocks omitted (no error)
- [x] Network error / non-200 -> usage blocks omitted, cached failure
- [x] Usage shown in blue color, magenta if >80%
- [x] Tests: response parsing, cache TTL, error handling
