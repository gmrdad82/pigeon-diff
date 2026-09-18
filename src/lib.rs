// The Diff plugin for Done: a run's changes in its workspace, drawn as a diff in a
// sheet of its own. `diff.rs` parses the text the desk's `run-diff` answers, `sheet.rs`
// lays it out as lines under the tree's bounds, and `guest.rs` is the component's
// glue — the command, the click, the door call, the tree; `context.rs` reads the
// desk's slot context. All but the glue are plain Rust and test on the host; the glue
// builds for `wasm32-wasip2` alone.
pub mod context;
pub mod diff;
pub mod sheet;

#[cfg(target_arch = "wasm32")]
mod guest;
