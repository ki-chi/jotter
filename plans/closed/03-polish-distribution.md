# Stage 3 — polish & distribution

Makes jOtter shippable to other people: configurable, debuggable, packaged, documented. Last stage; assumes Stages 1–2.

- [x] **Config file** `~/.config/jotter/config.toml`: image row cap (currently hardcoded `MAX_IMAGE_ROWS`), output viewport height, autosave interval, keybindings.
- [x] **Theme.** Syntect theme hardcoded to `base16-ocean.dark`; unreadable on light terminals. Config option + sane light default.
- [x] **Logging.** `--log <file>` via `tracing`; kernel wire errors currently collapse into one status-line string. Plus a message-history overlay (`:messages`-style).
- [x] **CI + packaging.** GitHub Actions (fmt, clippy, test), release binaries, `flake.nix` with package + devshell (promised in the MVP plan, `00-mvp.md`, missing).
- [x] **Tests.** Feature-gated integration test spawning a real `ipykernel` (kernel.rs is ~350 untested lines); UI snapshot tests with ratatui `TestBackend`.
- [x] **Docs.** Keybinding table in README, demo GIF (vhs), document the non-goals and known quirks below.
- [x] **Cell execution timing** shown in the gutter (optional; jupyterlab parity).

## Non-goals (whole effort)

ipywidgets (render a `[widget]` placeholder, nothing more) · multiple notebook tabs · attaching to existing/remote kernels (foreign connection files, no process ownership, interrupt can't work via signal) · saving image outputs to files (`savefig` in the notebook covers it) · jupytext/percent formats · kernel completions/inspection (`E`-to-nvim is the pressure valve) · Windows.

## Known quirks (documented, deliberate)

- Wide images crop at the right terminal edge instead of scaling (keeps the one-time Lanczos prescale, avoids render-time rescaling/aliasing). Revisit only if it bites; the fix is a width cap in the decode-time prescale + rebuild on resize.
- Images taller than `MAX_IMAGE_ROWS` (18) are downscaled once at decode time.

## Implementation notes (2026-09-24)
- Logging uses the `log` facade + a tiny file logger instead of tracing: `log` was already in the dep tree (zeromq stack logs through it), so `--log` captures dependency wire logs with zero new heavy deps.
- Config keybinding remapping deferred — knobs shipped: max_image_rows, max_output_rows, autosave_secs, theme. Add key remapping when someone actually wants a different map.
- Demo GIF ships as `assets/demo.tape` (vhs script) — recording needs a real terminal; run `vhs assets/demo.tape` to generate.
