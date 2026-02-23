---
title: filter in tui based on hex
created: '2026-02-23T20:28:02.111803'
status: done
done_at: '2026-02-23T22:54:22.568997'
---


Currently `compute_filtered()` in `src/tui/mod.rs` only fuzzy-matches the input against `todo.title()`. Hex IDs (e.g. `9a82`) are displayed in the list but not searchable.

## Change

In `App::compute_filtered()`, add an OR branch: match if the **ID starts with** the input (case-insensitive prefix match), in addition to the existing fuzzy title match.

```rust
.filter(|(_, t)| {
    t.id.starts_with(&self.input)
        || fuzzy_match(&self.input, t.title())
})
```

## Acceptance criteria

- Typing `9a` surfaces all todos whose ID starts with `9a`
- Typing a non-hex string still fuzzy-matches against title as before
- Prefix match on ID is case-insensitive
- No changes to the `fuzzy_match` function itself

## Files

- `src/tui/mod.rs` — `compute_filtered()` (line ~105)
