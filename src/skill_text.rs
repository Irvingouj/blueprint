pub const SKILL_TEXT: &str = r#"## Blueprint — Cross-Agent Context Injection Tool

You have access to `blueprint`, a CLI tool for loading implementation plans with all referenced files resolved inline.

### Commands

- `blueprint load <handle>` — Load a saved plan. Output includes the plan markdown followed by the full contents of all referenced files, resolved and embedded in code fences.
- `blueprint list` — List available blueprints in the current project.
- `blueprint list --global` — List globally available blueprints.

### Usage Pattern

When given a blueprint handle, run `blueprint load <handle>` to get:
1. The implementation plan (markdown body)
2. All referenced source files, URLs, and git diffs — fully resolved and appended under a "Referenced Files" section

This gives you complete context to execute or review the plan without needing to read files individually.

### Example

```bash
blueprint load refactor-auth
```

Output will contain the plan followed by a "Referenced Files" section with each file's contents in fenced code blocks.

### Piping

```bash
# Feed to another agent
blueprint load my-plan | codex --prompt -

# Copy to clipboard
blueprint load my-plan | clip

# Write to file
blueprint load my-plan > context.md
```
"#;
