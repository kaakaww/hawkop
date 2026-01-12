# Shell Completion Ordering - Deferred

## Decision: Wait for clap_complete upstream improvements

After evaluating the options for improving shell completion ordering in HawkOp, we've decided to **wait for upstream clap_complete improvements** rather than implementing custom shell scripts.

## Summary of Research

### Requirements
1. Show scan IDs in chronological order (most recent first)
2. Hide options unless `-<TAB>` is typed
3. Order: subcommands → arguments → options

### Root Cause
- The StackHawk API returns scans in correct chronological order
- **Shells re-sort completions alphabetically** - this is shell behavior, not a Rust issue
- `clap_complete` doesn't yet support ordering control or conditional option hiding

### Relevant clap Issues to Watch
- **[#4916](https://github.com/clap-rs/clap/issues/4916)**: "Completions shouldn't suggest options unless current token starts with `-`" - Closed, but feature not fully implemented
- **[#5058](https://github.com/clap-rs/clap/issues/5058)**: "Filter and order completions based on input and importance" - Open
- **[#3166](https://github.com/clap-rs/clap/issues/3166)**: Native completion engine stabilization tracking issue

### Alternative Crates Evaluated
All were either immature, didn't solve the shell-level ordering problem, or added significant complexity:
- `clap_dyn_complete` - Sparse docs, unclear benefit
- `completers` - Prototype, breaking changes expected
- `shell_completion` - Bash only, not production ready
- `carapace` - Written in Go

### Why Not Custom Shell Scripts
Custom shell scripts (~400-600 lines across bash/zsh/fish) would:
- Require manual updates when commands change
- Add maintenance burden
- Duplicate clap's command structure

## Action Items

- [ ] Monitor clap_complete native completion stabilization (#3166)
- [ ] Star/watch issue #5058 for ordering improvements
- [ ] Revisit when clap 4.x stabilizes native completions (estimated 2025)

## Current State

HawkOp completions work correctly but with shell-default alphabetical ordering. Users can continue using dynamic completions via:
```bash
# Dynamic completions (current)
source <(COMPLETE=bash hawkop)
source <(COMPLETE=zsh hawkop)
COMPLETE=fish hawkop | source
```
