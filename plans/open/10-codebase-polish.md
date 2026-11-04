# Stage 10 — codebase polish before 1.0

The code works and is well commented, but it has grown into a few very large files: `src/ui.rs` is 3,900 lines with a 560-line `draw`, `src/app.rs` 2,600 lines, `src/nvim.rs` 1,000, and the tests sit inline at the bottom of each. A contributor can't read one of them in a sitting, and a change to markdown rendering compiles next to the status line and the help overlay. This stage splits them along their responsibilities, tidies comments, and applies the basic principles (single responsibility, DRY, KISS, YAGNI) where the code visibly bends them. No behaviour changes, no new features.

## Rules for every step

- No behaviour change. `cargo test`, `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` pass after each commit; `make bench` before and after the stage shows the same numbers within noise.
- Mechanical moves are their own `refactor:` commits, separate from any rewrite, so a diff is either "moved" or "changed", never both.
- No new abstraction without two users: no trait with one implementation, no generic helper for one call site. Delete before adding.
- Keep the "why" comments (there are many good ones) and the `ponytail:` markers; drop comments that restate the code.

## Splits

- [ ] **`src/ui.rs` → `src/ui/`**: `mod.rs` (`Rendered`, `CellBlock`, the public API and the eviction budget), `wrap.rs` (`wrap_lines`, `Wrapped`, `RowOrigin`), `cell.rs` (`render_cell`, outputs, image and math entries, `InlineImage`, `KittyImage`), `markdown.rs` (`render_markdown`, `md_line`, links, tables, `expand_tabs`), `frame.rs` (`draw`: the layout pass, the visible lines, the image pass, the status line), `overlays.rs` (help, completion and signature popups, pager, logs, debug, marks sidebar, command line, `input()` box, `modal`/`centered`), `tests.rs`.
- [ ] **`src/app.rs` → `src/app/`**: `mod.rs` (`App`, `open`, state), `keys.rs` (`on_key` and the mode handlers), `ex.rs` (the `:` commands), `files.rs` (save, save-as, autosave path and sidecar, `check_disk`, recovery), `mouse.rs`, `kernel_events.rs` (`apply_kernel_event`, the run queue, `end_signal`), `tests.rs`. `cellops.rs`, `completion.rs` and `signature.rs` stay.
- [ ] **`src/nvim.rs` → `src/nvim/`**: `rpc.rs` (msgpack transport, `Rpc`, the notification thread), `session.rs` (`NvimSession`, `NvimCell`, the setup script), `keys.rs` (`key_to_nvim`), and `Backend` in `mod.rs`.
- [ ] `src/kernel.rs` is fine at 760 lines; split `resolve_kernelspec` into `kernel/discovery.rs` only if it grows.
- [ ] `draw` itself: the layout pass (heights, anchoring, render window, eviction) becomes a function returning the row table; the line materialisation, the image pass and the status line each become one. `draw` reads as five calls.

## Duplication and design

- [ ] The log viewer and the pager handle the same scroll keys with two copies of the match in `on_key_inner`: one scroll-keys helper over a `top`/`up` value.
- [ ] Three places salvage the text into the builtin editor when nvim dies (`on_key_edit`, `on_paste`, the write path): one `lose_nvim(e)` on `App`.
- [ ] `Backend` has a dozen match arms that only forward to the builtin or nvim cell: check which callers need the enum at all and whether a small trait with two implementations (the one case where it has two users) reads better than the forwarding.
- [ ] Every `unwrap` outside tests gets a reason or a `?`: there are six (`kernel.rs` parsing `"127.0.0.1"`, mutex locks, `app.rs` after an `is_some` check, two in `cellops.rs`).
- [ ] Error handling stays `anyhow` at the edges (`main`, `App`) and typed `Result`s in the library-like modules (`notebook`, `kernel`, `nvim`); check the boundary is where it says it is.
- [ ] Re-read every `ponytail:` marker and decide: still the right ceiling (keep), hit (fix), or wrong (delete). Today: filename cap, scratch sidecar per directory, output scan per update, per-line completion context, undo stack `remove(0)`, one formatter process per cell, full-buffer nvim readback, no operator+motion combos in the builtin editor.

## Comments and names

- [ ] Each module starts with a header that says what it owns and the invariants a reader must know (for `ui`: heights are exact once rendered and estimated before; for `nvim`: one request doubles as the flush barrier; for `notebook`: everything not touched is written back unchanged).
- [ ] Names say what, not how: `Rendered` holds the render cache, `CellBlock` one cell's rendered state, `Hit` a row's click target; rename the few that drifted (`Pick`, `starts`, `view`).
- [ ] Doc comments on public items; the config doc comments stay user prose since the docs are generated from them.

## How to tell it worked

No source file over about 1,000 lines and no function over about 150. `make bench` matches the numbers from before the stage. The CHANGELOG for the stage has only `refactor:` entries. A new contributor can find "where markdown tables are rendered" from the file names alone.
