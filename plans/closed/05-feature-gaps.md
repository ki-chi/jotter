# Stage 5 — feature gaps

What still stands between jOtter and a daily driver after the Stage 1–4 work, ordered by how often each gap bites in real use. Found in the 2026-09 repo review; all items implemented 2026-09-25.

Key choices made while implementing: completion triggers on `Tab` only after a word char (otherwise `Tab` indents); split is `Ctrl+\` inside the editor (plain terminals report it as `Ctrl+4`; `Ctrl+-` is terminal zoom, `Ctrl+s` is save-muscle-memory), merge is `M` (JupyterLab's Shift+M), clear is `c`/`C`, redo is `U`, save-as is `S`; restart now asks (`R` again: restart, `a`: restart & run all). Runs during kernel startup queue by cell id. Sidecars moved to the XDG state dir; old `.name.ipynb.autosave` files beside notebooks are no longer read and can be deleted. The recovery prompt shows a changed-cell count rather than a diff.

## Must-have

- [x] **Line wrapping.** `ui::draw` renders a bare `Paragraph` with no `.wrap()`, so long lines are silently cut off at the right edge. This hits markdown prose (a Jupyter paragraph is usually one source line), long code lines and wide outputs. Row accounting assumes one rendered row per `Line`, so wrap at render-cache time instead: split cached `Line`s to the body width in `render_cell` (rebuild on width change in `on_resize`), and keep `src_map`/hit rows pointing at the source line a wrapped row came from. The live editor cell needs the same soft wrap, plus cursor placement across wrapped rows (today the cursor pins to the right edge). Use display width (`unicode-width`, already a transitive ratatui dependency) rather than char count; this also fixes the `ponytail:` wide-char drift in `md_line`.
- [x] **Completion and inspection.** Tab completion via `complete_request` on the shell channel (popup under the cursor in insert mode, both editor backends). Shift+Tab docs via `inspect_request` (`text/plain` into an overlay like `M`). Both are plain request/reply on the existing shell pump; add a reply-routing map next to `running`.
- [x] **Create new notebooks and save-as.** `jotter new.ipynb` currently fails in `Notebook::open` when the file doesn't exist. A missing path should give an empty nbformat 4.5 notebook with one code cell and the resolved kernelspec in `metadata`, and only be written on the first `w`. Add a save-as command (`:saveas`-style prompt on the status line, reusing the search-input buffer).
- [x] **`update_display_data`.** Display handles (`display(..., display_id=True).update()`, `tqdm.notebook`, many progress libs) never update, because the handler ignores the message and `transient.display_id` is now stripped before storage. Keep a session-only `display_id -> (cell, output index)` map built from `display_data` transients in `kernel::nb_output` before stripping, and replace the matching outputs in place on `update_display_data`, across cells, as Jupyter does.

## Should-have

- [x] **Kernel-language awareness.** Syntax highlighting (`cell_ext`) and the `E` temp-file suffix are hard-coded to Python. Read `metadata.language_info.name`, then `metadata.kernelspec.language`, and map it through syntect's `find_syntax_by_token` (it already accepts names like `julia` or `r`).
- [x] **Clear outputs.** Clear the selected cell's outputs and all outputs (with undo). Common before committing a notebook.
- [x] **Restart & run all.** One key for `R` + `Ctrl+r`. Queue the run until the new kernel's `Event::Ready` arrives instead of failing with "no kernel".
- [x] **Split and merge cells.** Split at the editor cursor, and merge the selected cell with the one below. Both are single `UndoOp`s built from the existing insert/delete/`Source` ops.
- [x] **`yy` fills the cell register.** Today `yy` copies only to the system clipboard (OSC 52), so `yy` → `p` pastes whatever `dd` last deleted. Set `yanked` as well; keep the OSC 52 copy.
- [x] **Redo for cell ops.** `Ctrl+r` in notebook mode is taken by run-all; a free key (`U`) could redo. Undo already stores inverse ops, so redo means pushing the inverse of each undone op onto a redo stack, cleared on any new op.
- [x] **Autosave recovery.** On startup jOtter only reports that a sidecar exists. Offer "r: restore autosave · d: diff summary · any key: ignore", where restore loads the sidecar as a dirty buffer and never overwrites the file directly.
- [x] **Sidecar location.** `.<name>.ipynb.autosave` lands next to the notebook and shows up as untracked in every git repo (the jOtter repo itself has one). Either document a `*.ipynb.autosave` gitignore line, or move sidecars under `$XDG_STATE_HOME/jotter/` keyed by the canonical notebook path.

## Out of scope (unchanged non-goals)

ipywidgets (placeholder only), `text/html` rendering beyond the `text/plain` fallback, multiple notebooks/tabs, connecting to existing or remote kernels, Windows.
