# Stage 8 — render only what's visible

> **Closed 2026-10-20.** Done in 0.4.0 (f2c4659): the first frame costs the same for 14 and 70 cells (0.21 s, `make bench`) and memory follows the screen. The optional render-ahead item stays unneeded: no measured frame stall.

Opening a notebook renders every cell before the first frame, so startup and memory grow with the notebook's length. Measured with `make bench` (2026-10-05, median of 5): first content after 414 ms for the 14-cell showcase but 792 ms for the same cells five times over (70 cells), and 32 MiB vs 61 MiB of memory. euporie's startup doesn't grow with length at all. After this stage the first frame should cost the same for 14 or 700 cells, and memory should depend on what is on screen, not on what is in the file.

## Where the time goes today

- `Rendered::build` (`src/ui.rs`) calls `render_cell` for every cell up front: syntax highlighting for code, the markdown pass, LaTeX rasterisation (RaTeX + resvg) for every `$…$` and `$$…$$`, and for every image output a base64 decode, PNG decode, the height-cap Lanczos resize, recolouring and the terminal-protocol encode (`SlicedProtocol`).
- Every `CellBlock` keeps all of that for the whole session: highlighted lines, wrapped lines, and the encoded images.
- `ui::draw` computes each cell's start row from every cell's height on every frame (`starts`), so heights must be known for all cells; today that means fully rendered.

## Plan

- [x] **Split a block's height from its content.** A cell needs a row count for layout, scrolling, `G`/`5G`, search jumps and mouse hits, but its lines and images only when on screen. Give `CellBlock` a cheap "layout" state (height only) and the full "rendered" state.
- [x] **Exact or estimated heights without rendering.**
  - Source and text outputs: count wrapped rows at the body width with `unicode-width`, without highlighting (wrapping doesn't depend on colours).
  - Image outputs: read the PNG/JPEG header for the dimensions (`image::ImageReader::into_dimensions`, no decode) and apply the same height cap; exact.
  - Math: unknown until rendered; estimate (inline math is always one row; display math 2 rows, capped by `max_math_rows`) and correct when rendered.
  - Collapsed outputs and the output viewport (`max_output_rows`) cap heights as today.
- [x] **Render on demand.** In `draw`, render the cells that intersect the viewport plus a margin (one screen above and below), and keep the rest in the layout state. Mark the estimate as exact once a cell is rendered.
- [x] **Scroll anchoring.** When a cell above the viewport turns from estimated to exact height, shift `app.scroll` by the difference so the visible content doesn't jump. The same applies when `rebuild_cell` changes the height of a cell above the view (an output growing while you read further down).
- [x] **Evict far-away content.** Drop rendered lines and encoded images for cells more than a few screens away (keep heights), so memory stays bounded in long notebooks. With the kitty protocol, delete the evicted images' terminal-side copies too.
  - *Changed 2026-10-05:* evicting by distance (beyond four screens) re-rendered every cell on each sweep through a long notebook and re-sent every kitty image: `make bench` burst navigation went from 368 ms to 2945 ms for 414 keys on the 70-cell notebook, and a test counted 360 cell renders for 3 sweeps instead of 65. Eviction now keeps rendered content up to a memory budget (`RENDER_BUDGET`, 64 MiB of image pixels plus ~256 bytes per line) and then drops the least recently seen cells first, never the render window. Startup stays lazy; a notebook that fits the budget renders each cell once. Tests: `sweeping_a_notebook_renders_each_cell_once`, `eviction_drops_least_recently_seen_until_within_budget`.
- [ ] **Later, if needed: render ahead in the background.** If scrolling into a heavy cell (a large plot, much math) stalls a frame, render the margin cells on a worker thread and swap them in. Start synchronous and measure first.

## Things that must keep working

Search (`/`) works on sources and is unaffected; run-all with the selection following scrolls into cells as they run; streaming output and `update_display_data` rebuild only their cell; mouse hit-testing and `src_map` need the rendered state, so a click on a not-yet-rendered cell renders it first; resize (`on_resize`) invalidates heights and wrapping as today.

## How to tell it worked

- `make bench`: `startup_first_ms` for `bench-70` close to `bench-14`, and `rss_mib` for both close to each other.
- A unit test: a notebook of a few hundred cells builds without rendering more than the first screen's cells; scrolling to the end renders the last ones; the row of a cell above the view stays put when its estimate is corrected.
