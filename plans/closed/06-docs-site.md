# Stage 6 — documentation site

> **Closed 2026-10-20.** The site, the media rig, the generated reference and the release notes are in place. Two items were deferred, not dropped: PR preview deploys (when outside contributors send docs changes) and versioned docs (at 1.0, `starlight-versions`).

Replace the README-derived mdBook with a real documentation site, on the level of the Hyprland wiki: a landing page that sells the tool in ten seconds, a guided path from install to a productive first session, one page per feature with a short looping video, an exhaustive reference (every key, config option, CLI flag, file path), and troubleshooting for the problems people actually hit. After this stage the README is a short pitch plus links, and the site is the single source of truth.

Two rules shape everything below. **Media is generated, not hand-recorded**: every screenshot and clip comes from a script that drives a real jotter in a real terminal, so re-recording after a UI change is one command, not an afternoon. **Reference pages are generated or checked against the code**: key tables, config options and CLI flags are pulled from `HELP` (`src/ui.rs`), `Config` (`src/config.rs`) and clap `Args` (`src/main.rs`), and CI fails when docs and code drift apart.

## Stack

- [x] **Starlight (Astro) replaces mdBook.** Gives us, out of the box: Pagefind full-text search, dark/light themes, a sidebar with groups, `<Tabs>` (per-distro install), `<Aside>` callouts, card grids for the landing page, "edit this page" links, last-updated dates, and prev/next navigation. MDX lets pages embed `<video>` components. It builds to static HTML and deploys to GitHub Pages as today (free). Alternatives I rejected: mdBook (too bare for a product site, weak media and landing page), Docusaurus (heavier React stack, no advantage here), Material for MkDocs (in maintenance mode since 2025).
- [x] Site lives in `docs/` (`docs/src/content/docs/**.mdx`), Node pinned via `.nvmrc`/`package.json` `engines`, `npm ci && npm run build` in CI. Remove `book.toml`, the ANCHOR comments in `README.md`, and the mdBook step in `docs.yml`.
- [x] Brand: otter mascot from `assets/banner.png` as logo/hero, a favicon, an accent colour matching the mascot, a monospace font close to the one in the recordings so text in screenshots and prose look alike.
- [ ] Optional, later: PR preview deploys (Cloudflare Pages free tier, or the `rossjrw/pr-preview-action` subfolder trick on GitHub Pages). Worth it once others contribute docs.

## Media pipeline (the core of this stage)

vhs can't be the main tool: its terminal has no kitty graphics, so plots and LaTeX (jotter's headline features) fall back to halfblocks. Record the real thing instead.

- [x] **Headless recording rig**, pinned in a `devShells.docs` in `flake.nix` (sway, kitty, wf-recorder, grim, ffmpeg, a pinned font, Python with ipykernel + matplotlib + numpy + tqdm). The same shell runs locally (`nix develop .#docs`) and in CI (Nix installer action), so recordings match across machines.
  - `sway` with `WLR_BACKENDS=headless WLR_RENDERER=pixman` and a fixed output size (e.g. 1280×800, scale 1).
  - `kitty` with a checked-in `docs/media/kitty.conf` (font, size, colours, no cursor blink, `allow_remote_control yes`) so every frame is deterministic.
  - Keystrokes via `kitty @ send-key` / `kitty @ send-text` (real key events, so `Shift+Enter`, `Ctrl+\` and the kitty keyboard protocol work as they do for users).
  - `wf-recorder` for clips, `grim` for stills; `ffmpeg` post-processing.
- [x] **Scene scripts**: `docs/media/scenes/<name>.nu`, one per clip. A scene is a list of steps (`key`, `text`, `wait <ms>`, `wait-for <status-line text>`, `still <name>`) run by a single driver, `docs/media/record.nu`. Waiting on the status line or on the kernel going idle, instead of on fixed sleeps, is what keeps recordings stable when CI is slow. `record.nu <scene>` records one clip, `record.nu --all` records everything.
- [x] **Fixture notebooks** in `docs/media/notebooks/`: small, purpose-built (a LaTeX-heavy one, a plotting one, a tqdm one, a long-output one, a broken-kernel one…). They are not `showcase.ipynb`, so a demo change never breaks a docs clip. Seeded RNG in every notebook, so plots come out identical each run.
- [x] **Output formats**: clips as AV1/VP9 `.webm` plus an H.264 `.mp4` fallback, each with a `.png` poster frame, ≤10 s, ≤1.5 MB. Embedded as `<video autoplay loop muted playsinline poster=…>` through one `<Clip name="…"/>` MDX component that also sets width, caption and a `prefers-reduced-motion` fallback to the poster. Stills as `.webp`. One GIF only, for the README hero (GitHub READMEs can't autoplay repo-hosted video).
- [x] **Where media lives**: media is not committed to git. It's generated and cached by the `docs` workflow, with the cache key hashing the scene scripts, fixture notebooks, `kitty.conf` and `src/**`, so docs-only edits deploy in a minute and UI changes re-record automatically. `record.nu` also runs locally for previews. If CI recording turns out flaky, fall back to committing media on an orphan `media` branch; never on `main`, where binaries would bloat every clone.
- [x] **Clip list** (each one feature, one idea):
  - hero: open notebook → run all → plots and LaTeX appear (landing page + README GIF)
  - vi editing inside a cell (motions, `ciw`, undo); nvim backend doing a macro
  - run + advance, run in place, run all with the queued/running gutter and the selection following
  - inline matplotlib plot; `z` zoom; `Z` native size
  - LaTeX: markdown cell with inline + display math, a latex cell
  - completion popup after `.`, `Shift+Tab` docs overlay
  - tqdm progress bar updating in place; `display(..., display_id=True).update()`
  - long output viewport: live tail, `[`/`]`, wheel, `o` collapse
  - `/` search, `n`/`N`
  - cell ops: `a`/`b`, `dd`/`p`, `J`/`K`, split `Ctrl+\`, merge `M`, `u`/`U`
  - `E` into `$EDITOR` and back
  - mouse: click to select, double-click to edit, wheel
  - `input()` prompt
  - crash → autosave restore prompt; quit with unsaved changes
  - `R` restart prompt → restart & run all; `Ctrl+c` interrupt of a `sleep` loop
  - `L` log viewer, `D` debug report
- [x] **Stills**: a theme gallery (one screenshot per syntect theme, via `XDG_CONFIG_HOME` pointed at a generated config), and a terminal compatibility gallery (kitty, WezTerm, Ghostty, foot, Konsole, Alacritty with halfblocks, inside tmux) from the same rig.

## Information architecture

Modelled on the Hyprland wiki's split into getting started / configuring / reference / help. Every guide page opens with the clip, then says what it does, how to use it, and which config keys affect it.

- [x] **Landing page**: hero clip, one-line pitch, `<Tabs>` install snippet, a feature card grid (each card has a still and links to its guide), "why jotter" against JupyterLab / euporie / VS Code (speed, lossless nbformat, terminal-native), and links to GitHub and releases.
- [x] **Getting started**
  - Install: `<Tabs>` per distro (deb, rpm, apk, Nix/flake, NixOS module snippet, cargo, static binary), verifying downloads against `SHA256SUMS`, and building from source.
  - Kernels: the kernel resolution ladder (venv / conda / notebook metadata / `--kernel`), installing ipykernel in a venv, uv, conda, and non-Python kernels (IJulia, IRkernel) with the language-awareness notes.
  - First notebook: a 5-minute tutorial that creates a notebook, runs cells, makes a plot, saves, quits. Each step gets a still.
  - Terminal setup: why kitty is best, what degrades elsewhere, the `Shift+Enter` caveat, tmux `allow-passthrough on`.
- [x] **Guides** (one page per feature area, matching the clip list): editing, running code, outputs & images, LaTeX, completion & docs, search & navigation, cell operations & undo, mouse, external editor, nvim backend (with the `g:jotter` guard snippet and the list of unsupported plugin UI), data safety (atomic saves, autosave, recovery, on-disk change detection), using jotter with git (clear outputs, nbstripout).
- [x] **Reference**
  - Keybindings: generated from `HELP`, grouped as in the `?` overlay, and searchable.
  - Configuration: every `Config` field with type, default, description and example, generated from `config.rs` doc comments.
  - CLI: generated from clap (also shipped as a man page, see below).
  - Files & paths: config, autosave/state dir, logs, and how XDG vars move them.
  - nbformat compatibility: what round-trips losslessly, which output MIME types render, and what is shown as a placeholder (widgets, HTML).
  - Terminal compatibility matrix: graphics protocol, keyboard protocol, mouse, true colour, for each terminal, with the gallery stills.
- [x] **Help**: troubleshooting (no kernel found, kernel dies on start, images not shown, blurry images, keys not recognised, nvim plugins misbehaving, slow first LaTeX render), an FAQ, the known limits and non-goals (moved from the README, with the reason for each), how to report a bug (attach the `D` report and a `--log`).
- [x] **Project**: changelog, contributing (dev shell, tests, how to record media), architecture overview (event loop, render cache, kernel pump; one diagram), license.

## Keeping docs honest (generation + CI checks)

- [x] `jotter --dump-docs <dir>` is a hidden clap flag that writes `keys.md` (from `HELP`), `config.md` (from `Config`) and `cli.md` (via `clap-markdown`). For `config.md`, doc comments aren't available at runtime, so either move them into a `const CONFIG_DOCS: &[(&str, &str)]` next to the struct, or derive them with a small `schemars` schema. Pick whichever is the smaller diff when implementing. The docs build runs `--dump-docs` first; generated files are gitignored.
- [x] A unit test asserts every `Config` field has a docs entry and every `HELP` row survives into `keys.md`, so a new option or key can't ship undocumented.
- [x] A link checker (`starlight-links-validator`) fails the build on dead internal links; external links are checked weekly by a scheduled job, not on every push.
- [x] Man page: `clap_mangen` output `jotter.1`, generated in the release job and shipped in the tarball, deb, rpm and apk (`/usr/share/man/man1/`). Shell completions (`clap_complete`: bash, zsh, fish, nushell via `clap_complete_nushell`) shipped the same way.

## Release notes

- [x] `git-cliff` (the repo already uses conventional commits) generates `CHANGELOG.md` and the GitHub release body, replacing `generate_release_notes`. The docs changelog page renders `CHANGELOG.md`.
- [ ] Versioned docs are deferred until 1.0. Until then the site tracks `main`, and pages mark new behaviour with a "since vX.Y" badge. At 1.0, add `starlight-versions` with one snapshot per minor release.

## README after this stage

Pitch, hero GIF, a three-line install, one screenshot, and links to the docs sections. Target ≤60 lines. Everything else moves to the site.

## Implementation notes (2026-09-25)

- Steps 1–2 done on branch `docs-site`: Starlight site with 31 pages (landing, getting started, 13 guides, reference, help, project), all written against the current code rather than the README.
- The generator is `jotter --generate <DIR>` (hidden flag) rather than `--dump-docs`: it writes the three reference pages plus `man/jotter.1` and `completions/` (bash, zsh, fish, nushell). No `clap-markdown`: the CLI table is ~20 lines over clap's own `Command` API.
- Config docs come from the `///` comments on `Config` (read through `include_str!`), so the struct stays the only source; those comments were rewritten as user-facing prose. The drift test checks every field has one. A HELP drift test would be tautological (the page is generated from HELP), so there isn't one.
- `<Clip name="…"/>` renders nothing until `docs/public/media/<name>.webm` exists, so pages already reference their clips; step 3 makes them appear.
- Release tarballs, deb/rpm/apk and the Nix package ship the man page and completions.
- The terminal compatibility table is from each terminal's documented support, not yet tested; step 4's gallery verifies it.
- Step 3 done (2026-09-27): rig + hero, run-all, plots clips; CI records them (run 36275761599) with identical output to local. Lessons: capture at 1920x1200 scale 2 and set matplotlib dpi 200 or plots look half-size; `wf-recorder -D` or it hangs on a static screen; kitty's `send-key shift+x` drops the shift, so single characters go through `send-text`; each scene gets `/tmp/jotter-demo/.venv` (kernelspec copied in) so the status line shows a project-like path and global kernels never leak in. Clips come out 7–13 s and 120–270 kB, well inside budget.
- Step 4 clips done (2026-09-27): all 16 recorded and embedded; 7–14 s each (the first-notebook tour 29 s), 40–260 kB. The mouse clip is dropped: headless sway draws no pointer into captures (tried a cursor theme, kitty's mouse_hide_wait, a wlrctl virtual pointer), and a faked cursor overlay isn't worth it for a one-table page. Scenes with short notebooks call `height` to shrink the screen. Recording the recovery clip exposed a real bug, fixed: pending questions were cut off before their choices in 80-column terminals; they now take the whole status line.
- Plots now show in the terminal's colours (`recolor_plots`, on by default, display-only); example notebooks carry no styling.
- Galleries and step 5 done (2026-09-27): theme stills (7 themes, kitty set-colors per theme) and a terminal gallery (kitty, Ghostty, foot, WezTerm, Alacritty; tmux detection only). The table is now measured: jotter logs a `terminal:` line (graphics protocol, cell size, keyboard protocol) and the scene prints it. Corrections: WezTerm uses the iTerm2 protocol, Alacritty supports the keyboard protocol. Konsole, GNOME Terminal and xterm are listed as untested. git-cliff changelog + release notes; README cut to 48 lines with the hero as a GIF published by the docs build. FAQ/troubleshooting still lack real user reports.
- Also found: jotter aborts (SIGABRT, core dump) when its terminal is closed under it (seen when foot is killed). Checked 2026-09-29: the SIGHUP autosave is written first and the kernel exits within a second (`JPY_PARENT_PID`), so no work is lost. Cause: on exit `main` calls `ratatui::restore()`, which fails on the dead tty and reports it with `eprintln!`; stderr is the same dead tty, so `eprintln!` panics, ratatui's panic hook calls `restore()` again, that `eprintln!` panics too, and the panic-in-panic-hook aborts. Terminal's `Drop` does the same when it re-shows the cursor. Fixed 2026-09-29: `main` ends with `try_restore()`, an explicit `show_cursor()` and `mem::forget(terminal)`; closing the terminal now exits 0 with the autosave written. An external `kill -ABRT` leaving the terminal in jotter's modes is a non-goal (documented in Known limitations).
- Kernel stall while recording (fixed 2026-09-29): with completion-on-dot, a cell could stay at `In [·]` with the kernel idle (5 of 8 plots takes). Cause: ipykernel <= 7.3 can strand a shell request that arrives while it sends a reply (edge-triggered `ZMQ_FD`, fixed upstream in ipython/ipykernel#1529, merged 2026-08-13, unreleased); the popup's bursts of `inspect_request`s hit it. Confirmed by a second client unsticking the hung kernel, and 0/10 hangs with the upstream fix monkeypatched in. Workaround in `src/kernel.rs`: one shell request in flight at a time, with `stop_on_error` now enforced client-side (a failed run skips the runs queued behind it). 0/10 hangs after. The scenes no longer set `complete_on_dot = false`. ipykernel 7.4.0 (2026-09-29) ships the upstream fix; the workaround stays, because projects pin older ipykernels and `kernel_info` reports IPython's version, not ipykernel's, so jotter can't tell which kernels are fixed.
- Also found: the `input()` box is partly drawn over by an inline image directly above it (kitty image layer over the text overlay). Fixed in 7eef99d.

Checked 2026-10-20: the site has all the sections above (31 pages, generated reference, clips and galleries). Still open: PR preview deploys (optional) and versioned docs, which wait for 1.0.

## Order of work

1. Starlight skeleton + landing page + IA with stub pages; deploy (replaces mdBook).
2. `--dump-docs`, generated reference pages, drift tests, man page and completions.
3. Recording rig + driver + 3 clips (hero, run all, plots), to prove the pipeline in CI.
4. Remaining clips and stills; write the guide pages around them.
5. Troubleshooting/FAQ, terminal matrix, git-cliff changelog, README slimming.

## Out of scope

Translations; versioned docs before 1.0; a blog; hosted analytics (if we ever want it, use GoatCounter or Plausible; never Google Analytics); a custom domain (easy to add later with a `CNAME`).
