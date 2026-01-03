---
description: Add a new CLI command to HawkOp
---

Help me add a new command to the HawkOp CLI.

**Before designing, invoke the `cli-designer` skill** to ensure the command follows clig.dev and 12-Factor CLI principles.

## Questions to Answer

1. What should the command do?
2. What command name should be added?
3. What subcommands (if any) should it have?
4. What flags does it need? (prefer flags over multiple positional args)
5. What arguments does it need? (max 1 type of positional arg)

## Implementation Checklist

- [ ] Add help text with examples in `after_help`
- [ ] Support `--format json` for machine-readable output
- [ ] Use stdout for data, stderr for messages
- [ ] Return non-zero exit code on failure
- [ ] Add navigation hints (`→ hawkop next-command`)
- [ ] Test with pipes: `hawkop cmd | jq`

## Implementation Steps

1. Add API models in `src/client/mod.rs`
2. Add trait method to `StackHawkApi` trait
3. Implement in `src/client/stackhawk.rs`
4. Add display model in `src/models/display.rs`
5. Add CLI command enum in `src/cli/mod.rs`
6. Create handler in `src/cli/<command>.rs`
7. Wire up in `src/main.rs`

Follow patterns in `src/cli/` and the cli-designer skill's `patterns.md`.
