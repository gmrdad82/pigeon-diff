# pigeon-diff

The Diff plugin for Done, the orchestrator desk: a run's changes in its
workspace shown as a diff in a sheet of its own, so a review reads what
the implement round wrote without leaving the desk.

## What it does

A Task whose latest run has a git branch gets one verb on its tablet,
*Show the diff*. The verb opens a sheet: the Task and the run's name on
top, the totals (`2 files · +2 −1`), then what the run committed on its
branch since the Project's checkout and what still sits uncommitted in
its worktree, file by file — a header row (`path  +n −m`), the `@@`
lines, the hunks in monospace with git's own `+`/`-` at the head of each
line, added lines in the desk's `ok` tone and removed lines in `danger`.
A diff longer than the desk sends (2 MiB) ends with the desk's cut line;
one longer than a sheet may carry (4,000 lines or 60,000 bytes) ends with
how many lines stayed out. `Esc` closes the sheet.

The plugin cannot run git: it runs sandboxed, with no filesystem and no
process. The desk can, and answers the `pito:pigeon` door's `run-diff`
function — the Task's key in, one unified text out, bounded in time and
in bytes — which is all this plugin asks for. Its one capability is
`core:read`.

## How it is built

- `src/diff.rs` parses the unified text: `diff --git` headers, `---`/`+++`,
  `@@` hunks, `+`/`-`/` ` lines, `\ No newline at end of file`, binary
  notices, the `--stat` block, the desk's `# ` section lines and its `… `
  cut line. Pure Rust, no crate, a test per construct.
- `src/sheet.rs` lays the parsed diff out as lines under the desk's tree
  bounds (5,000 nodes, 65,536 text bytes).
- `src/guest.rs` is the component's glue: the `command` slot, the click,
  the door call, the tree. It builds for `wasm32-wasip2` alone; the two
  modules above test on the host with `cargo test`.
- `wit/` holds the plugin's world and byte-for-byte mirrors of the host
  interfaces it links (`wit/README.md`).

```
cargo test
cargo build --release --target wasm32-wasip2
```

The artifact is `target/wasm32-wasip2/release/pigeon_diff.wasm`; a tagged
release attaches it as `plugin.wasm` beside `plugin.toml` and
`SHA256SUMS`, and Done installs it from this repository by name,
verifying the hash before loading anything. Official: the owner of this
repository is the owner of the desks.

## State

The desk side of `run-diff` exists. The desk's plugin host does not yet
link a product world for a guest to import, so installing this release is
refused by the host's preflight ("component imports instance
`pito:pigeon/door@0.1.0`, but a matching implementation was not found in
the linker") until that seam ships in the host toolbox. Nothing in this
plugin changes for it.
