# Stage 7 — features for Jupyter and Neovim users

Candidate features after v0.3.0, for the target audience: people who live in Jupyter notebooks and edit in (Neo)vim. Collected 2026-10-05 from a review of the commits since v0.2.0 and the current key map (`HELP` in `src/ui.rs`) and config (`src/config.rs`); none of these exist yet. Ordered by value for effort within each group. Nothing here is committed to; pick per release.

## Small, high value

- [ ] **Copy a cell's output.** A key (e.g. `Y`) that copies the selected cell's text output to the system clipboard over OSC 52, as `yy` does for its source. Today the only way is `Shift`+drag, which breaks on soft-wrapped lines and on long outputs inside the scroll viewport. Join the `stream`/`text/plain` outputs; skip images.
- [ ] **Notify when a long cell finishes.** When a run that took longer than a threshold ends, send a desktop notification (OSC 9, OSC 777 for foot/urxvt) with the cell number, status and run time, and ring the bell as a fallback. Cells already record their run time (`b.elapsed` in `App::end_signal`). Config: `notify_after_secs` (default ~10, `0` off). The point: you switch windows during a 20-minute Monte Carlo run and get pinged.
- [ ] **Jump between markdown headings, and an outline.** Analysis notebooks are organised by headings, and Jupyter users get a table of contents. Keys to jump to the next/previous heading (`[`/`]` are taken by output scrolling, so e.g. `{`/`}`), and an overlay listing headings by level, like the `L` log viewer, where Enter jumps.
- [ ] **Remember where you were.** Per notebook, store the selected cell and scroll position in the state directory (next to the autosaves, keyed the same way) and restore them on open.

## Medium

- [ ] **Readable pandas DataFrames.** pandas sends a `text/html` table and an 80-column `text/plain` fallback, which jOtter shows today, so wide frames wrap or elide columns. Parse the `<table>` (pandas' HTML is regular: `thead`, `tbody`, `th`/`td`) into an aligned terminal table, using the full width and the output viewport's horizontal room. Fall back to `text/plain` on anything unexpected. DataFrames are among the most common notebook outputs.
- [ ] **Switch kernels inside jOtter.** A picker listing the kernelspecs the resolution ladder finds (venv, conda, global), which restarts on the chosen one and writes it to `metadata.kernelspec`. Today only `--kernel` at startup does this.
- [x] **Select a range of cells.** Shipped in 0.4.0 (`V`, 719d1ef). A cell-level visual mode (`V`, then `j`/`k`) whose selection `d`, `y`, `p`, run, merge and clear outputs act on as one undo step. Neovim users will reach for it; the undo stack already batches ops.
- [ ] **`jotter run notebook.ipynb`.** Run every cell headless and save the outputs in place (like `jupyter nbconvert --execute` / papermill), exiting non-zero on the first error. For cluster jobs: submit, then open the result later. Reuses the kernel launch, the run queue and atomic saves; mostly a CLI entry point without the TUI.
- [x] **Key remapping.** Decided for 1.0; planned in stage 09. Deferred since stage 3 ("add when someone actually wants a different map"). A `[keys]` table in `config.toml` mapping actions to keys, checked against `HELP` so the help overlay and the generated keybindings page show the user's map.

## Large, longer term

- [ ] **Static completion, signatures and docs before cells have run.** Today these come from the running kernel, so `np.` in a cell other than the import does nothing until the import cell ran (documented in the completion guide, "Before cells have run"). Two ways, both checked to work in a live kernel on 2026-10-05:
  - *Jedi with context (cheap):* when the kernel's answer is empty, retry completion with the code of the code cells above prepended (offsets shifted back), and get signatures from `jedi.Interpreter(...).get_signatures()` through a silent `user_expressions` request. Python only; strip the type annotations Jedi reads from numpy's stubs. Decided 2026-10-05 to keep the current behaviour for now.
  - *Language server (thorough):* run pyright/basedpyright or pylsp on a virtual document made of the notebook's code cells (LSP 3.17 notebook-document sync), for completion, signatures, hover and diagnostics without a kernel. What Neovim users get in `.py` files and would miss most here, and the biggest item on this list: an LSP client, an external program users must install, and mapping positions between cells and the document.

Checked 2026-10-20 against the code: none of the unchecked items exist yet (no `Y`, no notification escape, no heading keys, no stored position, no HTML table parser, no kernel picker, no `run` subcommand, no static completion).

## Out of scope (unchanged non-goals)

ipywidgets (placeholder only), rendering arbitrary `text/html` beyond tables, multiple notebooks/tabs, connecting to existing or remote kernels, jupytext/percent formats, Windows.
