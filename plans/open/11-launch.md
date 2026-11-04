# Stage 11 — public launch

jOtter is ready to be shown: docs site with comparison and limitations pages, installs through PyPI (`uv tool install jotter-tui`), crates.io, Nix and release packages, a custom GitHub social preview, and an "AI use" section in the README. What it lacks is people who know it exists (1 star, ~0 organic downloads as of 2026-10-22). This stage gets it in front of its audiences in a deliberate order: one big spike (Show HN), targeted posts that each have their own angle (Reddit, This Week in Rust, Ratatui), permanent listings and packages that keep bringing people in after the spike, and technical write-ups that give a reason to post again. Success is measured by where visitors come from, not by star counts alone.

## Rules for every step

- One channel at a time, a few days apart, so each post gets full attention in its comments and the traffic sources stay distinguishable.
- Never ask anyone to upvote. HN and Reddit both detect vote rings and bury or ban the post; sharing the link with friends is fine, asking for votes is not.
- Answer every comment in the first 3–4 hours. Thank critics, fix what they find, never argue. A fast fix released during the thread is the best possible reply.
- Claims stay measurable: link the speed numbers on the comparison page instead of saying "blazing fast". State the limitations before someone else does.
- AI use: the README says it plainly. If asked, answer matter-of-factly and point at the tests, CI and docs; never get defensive and never let the thread become about it.
- Only post where the rules allow it. Read each community's self-promotion and AI rules before posting there.

## Before launch day

- [x] **GitHub repo fields** (done 2026-10-22): `gh repo edit --description "A fast Jupyter notebook TUI with vim keys, inline plots and rendered LaTeX" --homepage "https://samox73.github.io/jotter/" --add-topic jupyter --add-topic jupyter-notebook --add-topic ipynb --add-topic tui --add-topic terminal --add-topic rust --add-topic ratatui --add-topic vim --add-topic neovim`
- [x] **HN account** (created 2026-10-22): create one at https://news.ycombinator.com/login now and use it for a few days (read, upvote, an occasional genuine comment). Brand-new accounts can get less visibility.
- [ ] **First keystroke swallowed when the terminal doesn't answer the startup queries** (found 2026-10-22 by the install tests): ratatui-image's `query_with_timeout` (`picker.rs`) reads stdin from a helper thread with a 1 s timeout; on timeout the thread stays blocked in `read()` and eats the next input (the first `:q` did nothing, the second quit). Real terminals answer, so it hits only silent ones (no reply within 1 s). Fix: report upstream, and/or do the graphics query ourselves with `poll` like `recolor::query` does.
- [x] **Install paths, from scratch** (tested 2026-10-22, all pass after two doc fixes: the real minimum Rust is 1.90, not 1.85; openSUSE's `zypper` needs `--allow-unsigned-rpm`. Commit those, they ship with the next release): on a clean machine or container, follow the README and the install page exactly for `uv tool install jotter-tui`, `uvx --from jotter-tui jotter`, `cargo install jotter-tui --locked`, `nix run github:samox73/jotter`, the static tarball and the `.deb`. A broken install in a launch thread costs more than any missing feature.
- [ ] **Predictable questions, answers ready** (most are already on the docs site; link, don't retype):
  - Why not euporie / JupyterLab / VS Code / molten-nvim? → comparison page.
  - Windows? → WSL. Widgets, HTML outputs, remote kernels? → limitations page; run jOtter on the remote machine over SSH.
  - Built with AI? → README "AI use" section, the tests and CI.
  - Why Rust / why not a Neovim plugin? → one honest sentence each.
- [x] **Awesome lists** (opened 2026-10-22): awesome-ratatui → https://github.com/ratatui/awesome-ratatui/pull/525, awesome-jupyter → https://github.com/markusschanta/awesome-jupyter/pull/189. Check on them weekly; answer review comments.

## Launch day: Show HN

Hacker News (https://news.ycombinator.com) is Y Combinator's link aggregator, the most-read news site among programmers. "Show HN" is the category for things you made that people can try right away; a post that reaches the front page brings thousands of visitors in a few hours. Read https://news.ycombinator.com/showhn.html first.

- [ ] **Day and time**: Tuesday to Thursday, 8–10 am US Eastern. Europe leaves summer time on 2026-10-25 and the US on 2026-11-01, so that is 13:00–15:00 Vienna in the week of 2026-10-26 and 14:00–16:00 from 2026-11-02 on. Keep the next 4 hours free.
- [ ] **Submission**: URL `https://github.com/samox73/jotter`, title along the lines of `Show HN: jOtter – a Jupyter notebook TUI with vim keys and inline plots`. No superlatives; HN strips or penalizes marketing words.
- [ ] **First comment, posted right after submitting** (draft it in the session, in your own voice):
  - Who you are and why you built it (notebooks on remote machines over SSH, alt-tabbing to a browser, wanting vim keys and no lag).
  - What's different: lossless `.ipynb` round-trip, inline plots via kitty/sixel/iTerm2, LaTeX without a TeX install, embedded Neovim, finds the project's environment by itself; latency measured against euporie (link).
  - What it deliberately doesn't do (link the limitations page).
  - One line on how it's built (Claude Code, see README).
  - What feedback you want.
- [ ] **During the thread**: answer everything, note every bug report as a GitHub issue, and if a fix is small, ship it as a patch release the same day and say so in the thread.
- [ ] **If it doesn't take off**: HN allows a repost of something that got no attention, after a while; the mods also re-up good posts through the "second chance pool". Don't repost the same week.

## The following two weeks: targeted posts

Upload the video directly to Reddit (not a link to the GIF), with a short text body and the install line. 2–3 days between posts.

| Where | Angle | Check first |
| --- | --- | --- |
| r/rust | How it's built: ratatui, the latency work, byte-for-byte `.ipynb` round-trip, pure-Rust ZeroMQ | Self-promotion and AI rules, flair |
| r/neovim | Embedded Neovim running your own config inside each cell | jOtter isn't a plugin; check whether they allow tools |
| r/commandline | The general pitch with the hero video | Rules |
| r/Python | Notebooks in the terminal, `uv tool install jotter-tui` | Showcase posts need their template (what it does, target audience, comparison) |
| r/HPC, r/JupyterNotebooks | Jupyter on a cluster over plain SSH: no port forwarding, plots inline | Rules; small subs, low effort |
| r/NixOS | `nix run github:samox73/jotter` | Rules |

- [ ] **This Week in Rust**: PR to https://github.com/rust-lang/this-week-in-rust adding a link under "Project/Tooling Updates" in the current draft (the repo README says where). It links to an article or announcement, so point it at the release page or the first write-up.
- [ ] **Ratatui community**: post in the showcase channel on their Discord and on https://forum.ratatui.rs. Optionally add the "Built With Ratatui" badge to the README.
- [ ] **Your own circles**: your research group, and the user docs or wiki of the HPC cluster you use ("Jupyter notebooks over SSH without port forwarding" is a page they might actually want).
- [ ] **Lobsters** (https://lobste.rs) is invite-only and a good audience for this; ask someone you know there for an invite. Their rule: no more than a quarter of your submissions may be your own work.

## Packages

Each of these is also a discovery channel: package indexes, Repology and "new packages" feeds.

- [ ] **AUR**: `jotter-bin` (the static tarball from the release, trivial) and optionally `jotter` (built with cargo). Needs an AUR account and SSH key. Then replace "an AUR package is planned" on the install page.
- [ ] **nixpkgs**: `pkgs/by-name/jo/jotter/package.nix` with `rustPlatform.buildRustPackage`, based on the flake's package. No popularity threshold.
- [ ] **Homebrew**: own tap first (`samox73/homebrew-tap`, `brew install samox73/tap/jotter`) using the release tarballs. homebrew-core has a notability threshold (check "Acceptable Formulae"); submit there once it's met.
- [ ] **conda-forge** (lower priority): a recipe in `conda-forge/staged-recipes` that builds from source with the Rust compiler. Jupyter users who live in conda would find it next to their kernels.
- [ ] Update the install page and README for each one that lands.

## Write-ups

The posts that keep working are technical stories, not announcements; each one is a reason to post on HN (as a normal submission, not Show HN), r/rust and Lobsters again, and it links back to jOtter.

- [ ] Decide where they live: a blog section on the docs site (Starlight has blog plugins) keeps everything in one place.
- [ ] Candidates, strongest first:
  - **Keystroke latency in a TUI**: how you measured it, jOtter at ~1 ms per key, and the finding that kitty adds ~10 ms while placeholder images are on screen (re-measure before publishing). Terminal authors and TUI developers will care.
  - **Saving `.ipynb` byte-for-byte like Jupyter**: what Jupyter's serializer actually does and how you matched it.
  - **LaTeX in a terminal without TeX**: the rendering pipeline from source to kitty graphics.
- [ ] One write-up a month is plenty; release posts in between (r/rust and TWIR accept them) when a release has something people will notice.

## Later, when eligible

- [ ] **awesome-tuis** (https://github.com/rothgar/awesome-tuis): repos must be at least 6 months old, enforced by CI. Eligible from 2027-03-17. Entry under "Development", next to euporie: `- [jOtter](https://github.com/samox73/jotter) A fast Jupyter notebook TUI with vim keys, inline plots and rendered LaTeX`.
- [ ] **awesome-rust** (https://github.com/rust-unofficial/awesome-rust): needs 50 GitHub stars or 2,000 crates.io downloads. Template: `[samox73/jotter](https://github.com/samox73/jotter) [[jotter-tui](https://crates.io/crates/jotter-tui)] - DESCRIPTION`, probably under "Applications → Text editors" or "Productivity"; add the CI badge as their CONTRIBUTING asks.

## Measuring

- [ ] **GitHub Insights → Traffic**: referring sites and popular content. GitHub keeps only 14 days, so write the numbers down (a small table in this file) within a week after each post.
- [ ] Stars over time, issues and discussions opened by people other than you (the best signal), PyPI downloads (pypistats.org) and crates.io downloads. Registry counts include bots: crates.io showed ~10 bot downloads on every release day before anyone knew about jOtter.
- [ ] After two weeks: which channel brought the most visitors and the most issues? Do more of that one.

## Log

| Date | Event | Stars | Views (14 d, unique) | Top referrers (views) |
| --- | --- | --- | --- | --- |
| 2026-10-22 | Before any posting | 1 | | |
| 2026-10-28 | awesome-ratatui PR merged; awesome-jupyter PR still open | 38 | 137 (58) | t.co 11, github.com 10, linkedin.com 9, youtube.com 7, docs site 5, techhub.social 3, Google 2, bsky.app 2 |
