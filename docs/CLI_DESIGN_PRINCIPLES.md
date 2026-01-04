# HawkOp CLI Design Principles

This document defines CLI design standards for HawkOp, based on [clig.dev](https://clig.dev) and [12-Factor CLI Apps](https://medium.com/@jdxcode/12-factor-cli-apps-dd3c227a0e46).

## Philosophy

> "CLIs are a fantastic way to build products. Unlike web applications, they take a small fraction of the time to build and are much more powerful."
> — Jeff Dickey

HawkOp aims to be a **human-first CLI** that remains composable with other tools. We optimize for:

1. **Discoverability** - Users learn through exploration, not documentation
2. **Composability** - Output works with pipes, grep, jq, and scripts
3. **Speed** - Fast startup, responsive feedback
4. **Empathy** - Helpful errors, sensible defaults

---

## The 12 Factors

### 1. Great Help is Essential

Help is the primary way users discover functionality. Without a UI to guide them, help must be exceptional.

**Requirements:**
- All these must display help:
  ```bash
  hawkop                    # List commands
  hawkop --help             # Full help
  hawkop -h                 # Full help
  hawkop <cmd> --help       # Command help
  hawkop <cmd> -h           # Command help
  ```
- `-h`/`--help` are **reserved** - never use for other purposes

**Help content must include:**
- Description of the command
- Description of all arguments and flags
- **Examples** - most referenced part of help; include even if "obvious"

**Implementation (clap):**
```rust
#[command(
    about = "Get scan details and drill down into findings",
    after_help = "EXAMPLES:\n  \
        hawkop scan get abc123              # Scan overview\n  \
        hawkop scan get abc123 alerts       # List all findings\n  \
        hawkop scan get abc123 40012        # SQL Injection detail\n  \
        hawkop scan get abc123 40012 uri-1  # Specific finding"
)]
```

**Shell completion** is another form of help - implement dynamic completions where valuable.

---

### 2. Prefer Flags to Args

Flags are self-documenting; positional arguments require memorization.

**Rule of thumb:**
- 1 type of positional arg → OK (e.g., `scan get <id>`)
- 2 types → suspect (consider flags)
- 3+ types → always use flags

**Bad:**
```bash
hawkop fork FROMAPP TOAPP  # Which is source? Which is destination?
```

**Good:**
```bash
hawkop fork --from FROMAPP --to TOAPP
```

**Exceptions:**
- Single obvious argument: `hawkop scan get <id>`
- Variable-length same-type args: `rm file1 file2 file3`

**For passthrough commands**, support `--` to stop flag parsing:
```bash
hawkop run --app myapp -- ./script.sh -a arg1
```

---

### 3. What Version Am I On?

Users need version info for debugging and support.

**Required:**
```bash
hawkop version      # Multi-command CLIs
hawkop --version    # Standard
hawkop -V           # Short form
```

**Include helpful debug info:**
```
hawkop 0.3.0 (darwin-arm64)
API: https://api.stackhawk.com
Config: ~/.config/hawkop/config.yaml
```

**Send version as User-Agent** for server-side debugging.

---

### 4. Mind the Streams

Proper stream usage enables composability.

| Stream | Purpose | Example |
|--------|---------|---------|
| stdout | **Data output** | Scan results, JSON, tables |
| stderr | **Messaging** | Warnings, progress, errors |

**Why this matters:**
```bash
hawkop scan list > scans.json    # JSON goes to file
# Progress bar still visible on stderr
```

**Rule:** If stdout is redirected, the user still sees stderr. Use this for:
- Progress indicators
- Warnings
- Spinners

**Implementation:**
```rust
// Data output
println!("{}", table);

// Status messages
eprintln!("Fetching scans...");

// Errors (via anyhow/thiserror)
return Err(anyhow!("API error: {}", msg));
```

---

### 5. Handle Things Going Wrong

Errors happen frequently in CLIs. Make them helpful.

**Great error message contains:**
1. Error code/type
2. Error title
3. Error description (optional but helpful)
4. **How to fix it**
5. URL for more info (optional)

**Bad:**
```
Error: EPERM
```

**Good:**
```
Error: EPERM - Invalid permissions on output.json
Cannot write to output.json, file does not have write permissions.
Fix with: chmod +w output.json
Docs: https://docs.stackhawk.com/hawkop/errors#eperm
```

**For unexpected errors:**
- Support `--debug` for full output
- Log to file for post-mortem (with timestamps, no ANSI codes)

**Implementation:**
```rust
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing API key\n\nRun 'hawkop init' to configure authentication.")]
    MissingApiKey,

    #[error("Invalid organization ID: {0}\n\nUse 'hawkop org list' to see available organizations.")]
    InvalidOrgId(String),
}
```

---

### 6. Be Fancy!

Modern CLIs should look good - but gracefully degrade.

**Use:**
- Colors to highlight important information
- Spinners for operations >1s
- Progress bars for known-length operations
- Box drawing for structure

**But respect the environment:**

| Condition | Behavior |
|-----------|----------|
| `NO_COLOR` set | Disable colors |
| `TERM=dumb` | Disable colors |
| `--no-color` flag | Disable colors |
| stdout not TTY | Disable colors, spinners, progress |
| stderr not TTY | Disable colors on stderr |

**Implementation:**
```rust
use std::io::IsTerminal;

fn should_use_color() -> bool {
    std::io::stdout().is_terminal()
        && std::env::var("NO_COLOR").is_err()
        && std::env::var("TERM").map(|t| t != "dumb").unwrap_or(true)
}
```

---

### 7. Prompt If You Can

Interactive prompts improve UX when appropriate.

**Guidelines:**
- Prompt when stdin is TTY and info is missing
- **Never require prompts** - always allow flag override
- Use confirmation for dangerous actions

**Good pattern:**
```rust
// If --org not provided and stdin is TTY, prompt
let org_id = match org_override {
    Some(id) => id,
    None if std::io::stdin().is_terminal() => prompt_for_org()?,
    None => return Err(anyhow!("--org required in non-interactive mode")),
};
```

**Confirmation for dangerous actions:**
```bash
$ hawkop config delete prod-config
Are you sure you want to delete 'prod-config'? [y/N]
```

---

### 8. Use Tables

Tables are the primary data display format.

**Rules:**
- **No borders** - they break parsing and add noise
- **One row = one entry** - enables `wc -l`, `grep`, piping
- **Headers by default** - can hide with `--no-headers`
- **Truncate to screen width** - unless `--no-truncate`

**Bad (with borders):**
```
+----------+--------+
| ID       | Status |
+----------+--------+
| abc123   | Done   |
+----------+--------+
```

**Good (borderless):**
```
ID        STATUS    APP              FINDINGS
abc123    Done      MyApp/prod       3 High, 5 Medium
def456    Running   OtherApp/staging --
```

**Support these table flags:**
| Flag | Purpose |
|------|---------|
| `--columns COL1,COL2` | Select columns |
| `--no-headers` | Hide headers |
| `--no-truncate` | Don't truncate wide content |
| `--sort COL` | Sort by column |
| `--filter KEY=VAL` | Filter rows |

**Always support `--format json`** - enables `jq` and scripting.

---

### 9. Be Speedy

CLI startup time directly impacts user experience.

**Benchmarks:**
| Time | Perception |
|------|------------|
| <100ms | Very fast (ideal for compiled languages) |
| 100-500ms | Fast enough - **aim here** |
| 500ms-2s | Usable but sluggish |
| 2s+ | Users will avoid your CLI |

**Measure regularly:**
```bash
time hawkop --help
time hawkop org list
```

**For slow operations:**
- Show spinner immediately
- Progress bar for known-length operations
- Stream results as they arrive when possible

---

### 10. Encourage Contributions

Open source CLIs benefit from community involvement.

**Checklist:**
- [ ] Open source with clear license
- [ ] README with overview and quick start
- [ ] CONTRIBUTING.md with:
  - How to run locally
  - How to run tests
  - Commit message format
  - Code quality expectations
- [ ] CODE_OF_CONDUCT.md
- [ ] Issue templates
- [ ] PR templates

---

### 11. Be Clear About Subcommands

HawkOp is a multi-command CLI (like git, npm, heroku).

**Empty invocation shows help:**
```bash
$ hawkop
HawkOp - Professional CLI companion for StackHawk

Commands:
  init      Initialize configuration
  org       Manage organizations
  app       Manage applications
  scan      View and manage scans
  ...
```

**Topic organization:**
```bash
hawkop scan list           # List scans
hawkop scan get <id>       # Get scan details
```

**Never reserve argument names as commands:**
```bash
# This is problematic if "help" could be a valid app name:
hawkop app help            # Is this help or an app named "help"?

# Prefer:
hawkop app --help
```

---

### 12. Follow XDG Spec

Store files in standard locations.

| Type | Unix | macOS | Windows |
|------|------|-------|---------|
| Config | `~/.config/hawkop/` | `~/.config/hawkop/` | `%APPDATA%\hawkop\` |
| Data | `~/.local/share/hawkop/` | `~/Library/Application Support/hawkop/` | `%LOCALAPPDATA%\hawkop\` |
| Cache | `~/.cache/hawkop/` | `~/Library/Caches/hawkop/` | `%LOCALAPPDATA%\hawkop\cache\` |

**Respect environment variables:**
- `XDG_CONFIG_HOME` overrides config location
- `XDG_DATA_HOME` overrides data location
- `XDG_CACHE_HOME` overrides cache location

---

## HawkOp-Specific Guidelines

### Standard Global Flags

All commands should support:

```rust
#[arg(long, global = true, env = "HAWKOP_FORMAT")]
format: OutputFormat,  // table, json

#[arg(long, global = true, env = "HAWKOP_ORG_ID")]
org: Option<String>,

#[arg(long, global = true, env = "HAWKOP_CONFIG")]
config: Option<String>,

#[arg(long, global = true, env = "HAWKOP_DEBUG")]
debug: bool,
```

### Pagination Flags

For list commands:

```rust
#[arg(long, short = 'l', default_value = "25")]
limit: usize,

#[arg(long, default_value = "0")]
page: usize,

#[arg(long)]
all: bool,  // Fetch all pages
```

### Filter Patterns

Use consistent filter flag names:

```rust
#[arg(long, short = 'a')]
app: Option<String>,      // Filter by app

#[arg(long, short = 'e')]
env: Option<String>,      // Filter by environment

#[arg(long, short = 's')]
status: Option<String>,   // Filter by status

#[arg(long)]
since: Option<String>,    // Date filter (30d, 7d, 2024-01-01)
```

### Navigation Hints

After displaying data, suggest next actions:

```rust
eprintln!();
eprintln!("→ hawkop scan get {} alerts", scan_id);
```

### JSON Output Format

Wrap JSON with metadata for consistency:

```json
{
  "data": [...],
  "meta": {
    "total": 150,
    "page": 1,
    "limit": 25,
    "has_more": true
  }
}
```

---

## Checklist for New Commands

Before merging a new command, verify:

### Must Have
- [ ] `-h`/`--help` works with useful description
- [ ] Examples in help text
- [ ] stdout for data, stderr for messages
- [ ] Exit code 0 on success, non-zero on failure
- [ ] `--format json` produces valid JSON
- [ ] Works in non-TTY environments (piping, scripts)

### Should Have
- [ ] Actionable error messages with fix suggestions
- [ ] Respects `NO_COLOR` and `--no-color`
- [ ] Tables are borderless, one entry per row
- [ ] Navigation hints for drill-down commands
- [ ] Pagination support for list commands

### Nice to Have
- [ ] Tab completion for dynamic values
- [ ] Progress indicator for slow operations
- [ ] `--quiet` flag for minimal output
- [ ] Streaming output for large datasets

---

## References

- [Command Line Interface Guidelines](https://clig.dev) - Comprehensive CLI design guide
- [12 Factor CLI Apps](https://medium.com/@jdxcode/12-factor-cli-apps-dd3c227a0e46) - Jeff Dickey's principles
- [clap Documentation](https://docs.rs/clap/latest/clap/) - Rust CLI framework
- [NO_COLOR](https://no-color.org) - Color disabling standard
- [XDG Base Directory Spec](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html)
