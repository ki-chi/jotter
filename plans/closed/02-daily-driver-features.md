# Stage 2 — daily-driver features

What you miss most in real use once Stage 1 (correctness & data safety) makes jOtter trustworthy. Generic-notebook scope: nothing here assumes a particular workflow.

- [x] **Scrollable long outputs.** Display at most N rows (~15) of an output as a viewport; full text stays in the model. Wheel-over-output scrolls it (add `HitKind::Output` to the hit map), one key scrolls for keyboard-only use, `o` collapses/expands the output entirely. Pairs with the Stage-1 cap: cap bounds memory, viewport bounds display.
- [x] **Run-all / run-above / run-below**, with a visible queued/running indicator per cell.
- [x] **Search** across all cell sources (`/`, `n`/`N`) and goto cell (`5G`).
- [x] **More MIME types.** `image/svg+xml` via resvg (already a dependency), `image/jpeg`/`gif` (image crate decodes them already — check the MIME dispatch), `text/markdown` outputs; `[text/html]` placeholder when HTML is the only rep.
- [x] **Markdown completeness.** Links, emphasis, tables, numbered lists; embedded images: cell `attachments` and local file paths.
- [x] **`input()` support** via the stdin channel (`allow_stdin: true` + a prompt line in the status bar), or at minimum a clear error message instead of a kernel-side exception.
- [x] **Cell-op undo.** Small undo stack for notebook-level ops (delete, move, type toggle, paste); `dd`→`p` is the only recovery today.
