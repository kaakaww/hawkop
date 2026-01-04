---
description: Run full quality check (fmt, clippy, test)
---

Run a comprehensive quality check on the HawkOp codebase:
1. Format the code with `make fmt`
1. Check formatting with `cargo fmt --check`
2. Run lints with `make lint`
3. Run tests with `make test`

Report all issues found and suggest fixes if needed.
