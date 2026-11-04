# Stage 1 — correctness & data safety

Blockers for calling jOtter production ready: a generic notebook TUI must not lose work, must not corrupt saves, and must render core Jupyter output semantics correctly. Do this stage first; move the file to `plans/active/` when work starts.

- [x] **Output pipeline: cap + coalescing.** Merge consecutive same-name stream outputs into one output object (what jupyter frontends save); cap the per-cell buffer (last ~10k lines). Today every chunk is a separate output pushed in `app.rs` `Event::Output`, each triggering a full `rebuild_cell` — O(n²) rendering and bloated saved JSON.
- [x] **Stream semantics: `\r` and `clear_output`.** `\r` within stream text overwrites the current line (tqdm, `print(end="\r")`); handle the `ClearOutput` iopub message (currently dropped in `kernel.rs` `translate_iopub`), including `wait=True` (clear on next output). Core Jupyter behavior, not a feature.
- [x] **Crash/quit safety.** `q q` on dirty discards silently → replace with save/discard/cancel prompt. Autosave sidecar (`.<name>.ipynb.autosave`) on a timer or after each execution; offer recovery on open; remove on clean save/quit.
- [x] **External-change detection.** Record mtime on open/save; before `w`, if the file changed on disk, warn instead of clobbering (jupyterlab, git checkout).
- [x] **Windowed rendering.** `ui::draw` clones every rendered line of every cell each frame and scroll caps at `u16::MAX` lines. Render only the visible window; drop the cap.
- [x] **Resize robustness.** On `Event::Resize` reflow is fine but image protocols keep stale cell dimensions; on font-size change everything drifts. Rebuild image protocols (or all of `Rendered`) on resize.
- [x] **fsync in atomic save** before the rename (small; closes the power-loss window).

Suggested order: the two output-pipeline items first — they touch the same code path (`kernel.rs` translate + `app.rs` `Event::Output`) and are one coherent change.
