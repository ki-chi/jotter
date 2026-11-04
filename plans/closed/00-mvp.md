# jOtter — a fast Jupyter notebook TUI in Rust 🦦

> **Archived 2026-09-25.** The original MVP plan, kept for history; superseded by stages 01–05 in this directory. Several "non-goals" below have since shipped (completion/inspection, `input()`, the config file, LaTeX rendering) — the README is the current feature reference.

Single goal: open a real `.ipynb`, edit it with vi keys, execute cells on a Jupyter kernel, see text + image outputs inline — with keystroke latency so low that 100 Hz key repeat is a non-event.

## Non-goals (MVP)

ipywidgets, HTML/LaTeX output rendering (plain-text fallback), kernel completions/inspection, `input()`/stdin_channel support, multiple notebooks/tabs, remote/existing-connection-file kernels, a config file. All v2 candidates; none block the core loop.

## Stack

| Concern | Crate | Note |
|---|---|---|
| TUI framework | `ratatui` + `crossterm` | enable crossterm's kitty keyboard protocol → Shift+Enter works |
| Async runtime | `tokio` | required by jupyter-zmq-client |
| Kernel protocol | `jupyter-zmq-client` + `jupyter-protocol` | ex-runtimelib; handles ZMQ channels, HMAC signing, kernelspec discovery, kernel launch |
| Cell editor | `tui-textarea` | vendor its `vim.rs` example (~400 lines) for modal editing |
| Images | `ratatui-image` + `image` | kitty graphics protocol; decode base64 PNG from `display_data` |
| Syntax highlight | `syntect` | Python + markdown, once per visible line, cached |
| ANSI in outputs | `ansi-to-tui` | tracebacks and colored stream output |
| Serialization | `serde` + `serde_json` | ipynb is plain JSON |
| Errors/CLI | `anyhow`, `clap` (derive) | `jotter notebook.ipynb [--kernel phy]` |

Native dep: ZeroMQ. NixOS dev shell (`flake.nix`): `zeromq`, `pkg-config`, rust toolchain.

## Architecture

One binary crate, no workspace. Modules:

- `main.rs` — CLI, terminal setup/teardown (incl. panic hook that restores the terminal), tokio runtime.
- `notebook.rs` — nbformat 4.5 model. Cells as typed structs but with `#[serde(flatten)] extra: Map<String, Value>` on every level so unknown metadata round-trips losslessly — we must never destroy fields other tools wrote. Atomic save (write temp + rename). Cell ids preserved/generated.
- `kernel.rs` — owns the jupyter-zmq-client connection. Launch kernelspec by name (default from `--kernel`), spawn a tokio task that pumps iopub/shell messages into an `mpsc` toward the UI. Route replies to cells by `parent_header.msg_id`. Expose: `execute(code) -> msg_id`, `interrupt()`, `restart()`, kernel status (busy/idle) from `status` messages.
- `app.rs` — app state machine: mode (NotebookNormal / CellEdit), selected cell, scroll offset, dirty flag, per-cell output buffers, pending-execution map (msg_id → cell id).
- `ui.rs` — render: scrolling cell list, each cell = prompt gutter (`In [n]`) + source (syntect) + outputs (ansi-to-tui text, ratatui-image for PNGs). Status line: kernel state, mode, dirty, filename.
- `editor.rs` — tui-textarea wrapper + vendored vim bindings + the nvim escape hatch: leave alt screen, dump cell source to `$XDG_RUNTIME_DIR/jotter-cell.py`, spawn `$EDITOR`, read back, re-enter alt screen.
- `outputs.rs` — translate iopub messages (`stream`, `execute_result`, `display_data`, `error`) into renderable output items; image decode + protocol-picker cache.

Event loop: single `tokio::select!` over crossterm's async event stream and the kernel mpsc. Every branch mutates state then draws once. No timers, no polling — idle CPU is 0%.

## Keys (notebook normal mode)

`j/k` select cell · `J/K` move cell · `Enter`/`i` edit cell (vi modal inside; `Esc` in normal mode exits back) · `Shift+Enter` run + advance · `Ctrl+Enter` run in place · `a/b` insert cell after/before · `dd` delete cell (with yank buffer, `p` pastes) · `m` toggle code/markdown · `E` open cell in nvim · `w` save · `Ctrl+C` interrupt kernel · `R` restart kernel · `q` quit (confirm if dirty).

## Milestones

**M0 — read-only viewer.** cargo init, flake.nix dev shell, terminal setup with panic hook. Parse + losslessly re-serialize a real notebook (round-trip test: `open → save → json-equal`). Render cells with syntax highlighting, j/k + scrolling. *Exit test: open a real notebook from a collaborator, everything visible, `:w` produces byte-stable JSON modulo formatting.*

**M1 — execution.** Launch `phy` kernelspec, heartbeat, execute selected cell, stream text/ANSI outputs live into the cell, execution counts, busy/idle in status bar, interrupt + restart. *Exit test: run a loop that prints with `time.sleep`; output streams in live; Ctrl+C interrupts.*

**M2 — editing.** tui-textarea + vim bindings in-cell, nvim escape hatch, cell add/delete/move/type-toggle, atomic save, dirty tracking. *Exit test: full edit session on a real notebook, reopen in jupyter-lab, nothing lost.*

**M3 — images.** `display_data`/`execute_result` PNG → kitty graphics inline, cached per output; matplotlib figure from the `phy` kernel renders inline. *Exit test: `plt.plot(...)` shows a plot; scrolling doesn't ghost or flicker.*

**M4 — polish (optional).** Markdown cell rendering (rendered when not selected, raw when editing), keybinding cheatsheet popup (`?`), `--kernel` autodetect from notebook metadata.

MVP = M0–M3. Estimate: M0+M1 one weekend each, M2 the long pole (vendored vim bindings do most of it), M3 a day or two thanks to ratatui-image.

## Risks / known ceilings

- `tui-textarea` vim example is basic (no registers/macros/counts on operators) — that's the accepted ceiling; `E` → nvim is the pressure valve.
- Kitty graphics + scrolling regions can flicker; ratatui-image handles placement but test early in M3, not last.
- `jupyter-zmq-client` is young; if an API gap appears, `jupyter-protocol` types + raw `zeromq` is the fallback (more code, same design).
- Cell outputs can be huge (MB of stream text) — cap per-cell output buffer (e.g. last 10k lines) from day one.
