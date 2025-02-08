# Forge Pretty Diff

A Rust library for generating beautiful and readable diffs with syntax highlighting and line numbers.

## Features

- Line-by-line diff comparison
- Context-aware diff grouping
- Line numbers for both old and new versions
- Color-coded changes (additions in green, deletions in red)
- Emphasized change highlighting
- Support for missing newlines

## Usage

```rust
use forge_pretty_diff::PrettyDiffer;
use std::path::PathBuf;

let old_content = "line 1\nline 2\nline 3";
let new_content = "line 1\nmodified line\nline 3";
let path = PathBuf::from("example.txt");

let diff = PrettyDiffer::format(path, old_content, new_content);
println!("{}", diff);
```

## Output Format

The output includes:
- File name and path
- Line numbers for both old and new versions
- Color-coded changes:
  - Red (-) for deletions
  - Green (+) for additions
  - Dimmed for unchanged lines
- Emphasized highlighting for specific changes within lines
- Context lines around changes

## Dependencies

- similar: For diff generation
- console: For terminal styling and colors