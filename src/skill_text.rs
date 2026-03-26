pub const SKILL_TEXT: &str = r##"## Blueprint — Cross-Agent Context Injection Tool

Blueprint is a CLI tool for saving markdown documents with file/URL/git references, then loading them with all referenced content resolved and inlined. This lets you capture implementation plans with their full context.

---

### Core Concepts

A **blueprint** is a markdown file with YAML frontmatter that declares references to external resources. When loaded, the tool resolves all references and appends their contents under a "Referenced Files" section.

**Storage locations:**
- Project-level: `.blueprint/` directory (found by walking up to `.git` root)
- Global: `~/.blueprint/`

---

### Blueprint File Format

A blueprint file has two parts: YAML frontmatter between `---` delimiters, followed by markdown body.

```yaml
---
references:
  - src/main.rs                    # Bare string = file path
  - lib/utils.ts                   # Relative paths resolved from base_dir
  - type: file                     # Explicit file reference
    path: config/settings.toml
  - type: url                      # HTTP/HTTPS URL (fetched at load time)
    url: https://api.example.com/docs
  - type: git-diff                 # Git diff output
    path: src/                      # Path to diff (file or directory)
    range: HEAD~3..HEAD            # Git revision range
base_dir: /home/user/project       # Set automatically on save
saved_at: 2024-01-15T09:30:00Z     # Set automatically on save
---

# Your Plan Title

Implementation plan goes here in markdown...

## Steps
1. Modify the auth module
2. Update tests
```

**Reference Types:**

| Type | Syntax | Description |
|------|--------|-------------|
| Bare string | `src/main.rs` | Shorthand for file reference |
| `file` | `type: file, path: <path>` | Local file (absolute or relative to base_dir) |
| `url` | `type: url, url: <url>` | Fetched via HTTP GET at load time |
| `git-diff` | `type: git-diff, path: <path>, range: <range>` | Output of `git diff <range> -- <path>` |

---

### Commands

#### `blueprint save` — Save a blueprint

Save markdown content as a named blueprint.

```bash
# From file
blueprint save --handle refactor-auth --file ./plans/auth.md

# From inline content (use literal newlines, not escaped)
blueprint save --handle quick-fix "# Fix plan

1. Update config"

# Save globally (accessible from any project)
blueprint save --handle global-template --file template.md --global
```

**Behavior:**
- Injects `base_dir` (current working directory) and `saved_at` (timestamp) into frontmatter
- Validates frontmatter structure and reference formats
- Warns if references list is empty
- Overwrites existing blueprint with same handle

#### `blueprint load` — Load and resolve a blueprint

Load a blueprint and resolve all references, outputting the complete context.

```bash
blueprint load refactor-auth
blueprint load global-template --global
```

**Output Format:**

```markdown
# Your Plan Title (original markdown body)

Implementation plan goes here...

---
## Referenced Files

### `src/main.rs`
```rust
// File contents here
```

### `https://api.example.com/docs`
```html
<!-- Fetched content here -->
```

### `src/ (HEAD~3..HEAD)`
```diff
// Git diff output here
```
```

**Resolution behavior:**
- File references: Read relative to `base_dir` or as absolute paths
- URL references: HTTP GET with 30s timeout
- Git-diff references: Run `git diff <range> -- <path>` with 15s timeout
- Failed resolutions: Output `[unresolved: <error>]` instead of content
- All resolutions happen concurrently for speed

**Language detection:** Code blocks use file extension for syntax highlighting:
- `.rs` → `rust`, `.ts` → `typescript`, `.py` → `python`, `.go` → `go`, etc.
- URLs → `html`, git diffs → `diff`

#### `blueprint list` — List available blueprints

```bash
blueprint list              # List project-level blueprints
blueprint list --global     # List global blueprints
```

Output: One handle per line, or "No blueprints found" message.

#### `blueprint skill` — Output this help text

---

### When to Use Blueprint

**Use blueprint when:**
- You need to capture a plan + its context for later execution
- You want to share a complete task context with another agent
- You're switching contexts and want to preserve state
- A task involves multiple files and you want atomic context loading

**Example workflow:**

```bash
# 1. Create a plan with references
blueprint save --handle fix-login --file - << 'EOF'
---
references:
  - src/auth/login.ts
  - src/auth/session.ts
  - type: git-diff
    path: src/auth/
    range: main..feature/new-auth
---

# Fix Login Flow

The login function needs to handle expired sessions gracefully.
See the diff for current changes, then update session handling.
EOF

# 2. Later, load complete context
blueprint load fix-login

# 3. Or pipe to another agent
blueprint load fix-login | llm --system "You are a code reviewer"
```

---

### Error Handling

| Error | Cause | Resolution |
|-------|-------|------------|
| `Blueprint 'X' not found` | Handle doesn't exist | Run `blueprint list` to see available handles |
| `failed to read 'path'` | File reference not found | Check path is relative to base_dir or use absolute path |
| `HTTP request failed` | URL unreachable or invalid | Check URL, ensure network access |
| `git diff failed` | Invalid git range or path | Verify range syntax (e.g., `HEAD~1..HEAD`) and path exists |
| `timeout after 30s` | URL taking too long | Check network, consider using file reference instead |

---

### Best Practices

1. **Use descriptive handles**: `refactor-auth-2024-01` vs `plan1`
2. **Prefer relative paths** in references — they're portable across machines
3. **Keep base_dir intact** — it's set automatically and enables portability
4. **Use git-diff for active work** — captures work-in-progress on a branch
5. **Validate with list**: Run `blueprint list` before `load` to confirm existence
6. **Check warnings on save**: Empty references list produces a warning but still saves

---

### Quick Reference

```bash
# Save current plan
blueprint save --handle <name> --file <path> [--global]

# Save inline
blueprint save --handle <name> "# Markdown content"

# Load with all references resolved
blueprint load <name> [--global]

# List available
blueprint list [--global]
```
"##;
