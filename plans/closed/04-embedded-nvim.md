# Stage 4 — embedded nvim as the cell editor (experiment)

Goal: replace the vendored tui-textarea vim bindings with a real, headless `nvim --embed` acting as the *modal engine* — jotter keeps rendering everything (syntect highlighting, images, layout), nvim owns buffer text, cursor, mode, registers, macros, and operators. This is the vscode-neovim architecture: keys go in over msgpack-RPC, buffer/cursor/mode come back out. We never render nvim's screen grid.

Status: experiment. Success = `dw`, `ciw`, visual mode, counts, registers, `.` repeat, and macros all work inside a cell with no perceptible latency. Failure is cheap: the builtin editor stays as the fallback, so this can land incomplete behind a config flag or be abandoned without touching current behavior.

## Architecture

- **One nvim child per session**, spawned lazily on first cell edit: `nvim --embed --headless -u NONE -i NONE -n` (v1: clean config — predictable semantics; user config is M3). Killed on drop like the kernel child; on crash or missing binary, fall back to the builtin editor with a status message.
- **RPC**: msgpack-RPC over the child's stdin/stdout via the `nvim-rs` crate (tokio feature — our runtime already fits). If `nvim-rs` fights us, fallback is a hand-rolled loop on `rmpv` (~200 lines); decide in M0.
- **Input path**: crossterm `KeyEvent` → nvim key notation (`<Esc>`, `<CR>`, `<C-r>`, `<lt>`, …) → `nvim_input()`. Translator is ~50 lines + a table test.
- **Readback path**: `nvim_input` is queued asynchronously, so after each input send one follow-up request as a flush barrier and read: mode (`nvim_get_mode`), cursor (`nvim_win_get_cursor`), and buffer lines. Buffer lines via `nvim_buf_attach` change notifications (line-range diffs pushed to us) rather than full-buffer polls; the notification stream joins the main `tokio::select!` like kernel events.
- **Rendering**: nvim's buffer lines feed the exact render path the builtin editor uses today (syntect highlight of `editor.source()`, cursor drawn by jotter, mode chip in the status bar). Modal cursor shapes keyed off nvim's mode string.
- **Jotter-level keys stay intercepted** (checked against last-known mode before forwarding): `Esc` in normal mode commits + exits the cell (current muscle memory), `Shift+Enter`/`Ctrl+Enter` commit + run. Everything else goes to nvim verbatim — including `q`, which becomes real macro recording, replacing the stage-hint message while the nvim backend is active.
- **Backend selection**: `enum EditorBackend { Builtin(Editor), Nvim(NvimCell) }` behind `editor = "nvim" | "builtin"` in config.toml (default stays `builtin` until the experiment graduates). New module `src/nvim.rs`; `app.rs` talks to the enum through the same tiny surface the builtin exposes today (`source()`, `lines`, `cursor`, `mode`, `click(row, col)` → `nvim_win_set_cursor`).

## Milestones

- [x] **M0 — spike.** Done (`src/nvim.rs`, `cargo test -- --ignored nvim_spike`): measured **160 µs/key** round-trip (budget was 1 ms). Decision: **hand-rolled sync RPC on `rmpv`** (~120 lines, one tiny dep) instead of nvim-rs — and *blocking* calls straight from the key handler instead of joining the tokio select loop; at 2 round-trips/key there is nothing to overlap. Readback is one deferred `nvim_eval("[getline(1,'$'), getpos('.'), mode(1), getcmdtype()..getcmdline(), getpos('v')]")` that doubles as the flush barrier; full-buffer poll instead of `nvim_buf_attach` diffs (cells are tens of lines).
- [x] **M1 — wired in behind the flag.** Done: `enum Backend { Builtin, Nvim }` behind `editor = "nvim"`, key forwarding + readback, commit on exit/run, click-to-position, crash/missing-nvim fallback to builtin (with mid-edit text salvage + one auto-respawn for a stale parked session). `:w` maps to a cell commit via `buftype=acwrite` + `BufWriteCmd → rpcnotify`. Exit criteria (`dw`, `ciw`, registers, macros, counts, `.`) asserted in `nvim_spike`/`backend_intercepts`; full TUI driven end-to-end in a pty.
- [x] **M2 — visual mode + per-cell buffers.** Done: selection overlay (REVERSED) from `getpos('v')` + cursor for `v`/`V`/`^V`; one buffer per cell keyed by nbformat cell id, parked `NvimSession` hands off between edits, so undo history and marks survive cell switches (initial fill is undo-fenced via `undolevels=-1`).
- [x] **M3 — comfort.** Done 2026-09-25: user config loads by default (`nvim_user_config = false` opts out) with `g:jotter = 1` set via `--cmd` before init; cell buffers get `filetype`; clean mode sets `expandtab` so Tab never inserts a literal tab. `ext_messages` not needed so far. Original note: Load user config with `vim.g.jotter = 1` set (config flag `nvim_user_config = true`); `ext_messages` for prompts; `filetype=python` on cell buffers. *(Partially preempted: `:`/`/` already echo in the status bar via `getcmdline()` in the readback — no UI attach needed.)*

## Risks / known ceilings

- **Async input races**: `nvim_input` queues; a readback racing the queue shows stale state. *Resolved:* the deferred eval is the barrier — the spike asserts mode `no` right after `d`. If nvim sits in a blocking prompt (hit-enter), the readback times out (250 ms) and a fast `nvim_get_mode` keeps the mode fresh + probes liveness; state resyncs on the next key.
- **Blind cmdline in v1**: *resolved early* — `getcmdtype()..getcmdline()` rides in the per-key readback and shows in the status bar, so `:` and `/` are visible without any UI attach.
- **`:w`-in-nvim writes nothing**: *resolved* — `nvim_create_user_command` can't shadow `:w`; instead cell buffers are `buftype=acwrite` named `jotter://…`, and a `BufWriteCmd` autocmd `rpcnotify`s a commit.
- **`:q` quits the embedded nvim** (last window; quits can't be vetoed by autocmd): *resolved* — cmdline abbreviations rewrite the common spellings (`q`, `q!`, `qa`, `qall`, `quit`, `wq`, `wq!`, `wqa`, `x`, `exit`) into user commands that `rpcnotify` jotter, and `ZZ`/`ZQ` are `nnoremap`ed to the same; jotter treats them as "leave the cell" (commit, or discard for `:q!`/`ZQ`). Exotic spellings (`:1q`, `:quita`) still kill the child and land in the crash-fallback net: text salvaged into the builtin editor, fresh nvim on next edit.
- **Plugin chaos under user config** (M3 only): plugins expecting a UI misbehave headless; that's why `vim.g.jotter` gating + default off.
  - *2026-09-25, observed*: telescope (`<leader>sg`) opened its prompt float and became the current buffer; the per-key readback read *that* buffer, so the picker's text replaced the cell. **Resolved (guard, not support):** readback reads the cell buffer by number (`getbufline(bufnr)`), and when `bufnr('%')` or `win_gettype()` shows nvim elsewhere, a Lua reclaim closes every float (re-checking validity: telescope closes its companion windows itself) and other split, `stopinsert`s, re-shows the cell buffer, and jotter shows a status message. **Not supported:** drawing nvim UI — that needs `nvim_ui_attach` with `ext_multigrid`/`ext_popupmenu` and rendering grids ourselves; revisit only if pickers inside cells become a real need.
- **Latency budget**: 2–3 RPC round-trips per key on a local pipe (~100–300 µs) against the "100 Hz key repeat is a non-event" goal — M0 measures before we commit.
- **Process hygiene**: kill + reap the child on drop (reuse the kernel monitor-task pattern); `-i NONE -n` so no shada/swap files ever touch disk.

## Non-goals

Rendering nvim's grid (jotter stays the renderer — no ext_linegrid interpretation beyond ignoring it if a UI attach is needed for ext_cmdline) · nvim owning anything outside the open cell (notebook-level keys, cell ops, scrolling stay jotter's) · replacing `E` (the full-screen external-editor hatch stays) · Windows.

## Fallback ladder

If M0/M1 stall: swap the builtin editor for the `edtui` crate instead (ratatui editor widget with visual mode and `dw`/`ciw`-class operators built in) — a dependency swap that covers the most-missed features without an embedded process.
