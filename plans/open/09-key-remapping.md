# Stage 9 — key remapping

Every key is hard-coded: `on_key_normal`, `on_key_visual`, `on_key_prompt` and the editor-level intercepts in `src/app.rs` match on `KeyCode` directly, and the `?` overlay and the generated keybindings page come from the separate `HELP` table in `src/ui.rs`. Users who want `Space` to run, arrows instead of `j`/`k`, or their Neovim leader habits can't have them, and a 1.0 freezes whatever map ships. Decided 2026-10-20: remapping ships before 1.0.

## Design

- **One table of actions.** An `Action` enum for the notebook and cell-visual modes (`next_cell`, `prev_cell`, `run_advance`, `run_here`, `run_all`, `new_cell_below`, `delete_cell`, `yank_cell`, `toggle_output`, `zoom_image`, `save`, `quit`, `command_line`, `help`, ...), each with its default key sequence(s) and help text. The key dispatcher, the `?` overlay, `jotter --generate` (the keybindings reference) and config validation all read this table, so a key can't exist in one place and not the others. The existing drift test extends to it.
- **Config.** A `[keys]` table in `config.toml`, action name to key or list of keys, in vim notation: `next_cell = "<Down>"`, `run_advance = ["<S-CR>", "r"]`, `zoom_image = "zi"`. An empty string unbinds. Keys not mentioned keep their defaults.
- **Key notation.** `<C-o>`, `<S-CR>`, `<CR>`, `<Esc>`, `<Space>`, `<Tab>`, `<Up>`, single characters, and sequences (`gg`, `zt`, `dd`). One parser from notation to a `Vec<KeyEvent>`-like sequence, with tests; the reverse for showing keys in the overlay.
- **Sequences and counts.** Today `pending` holds one prefix char (`g`, `z`, `m`, `'`, `d`, `y`). It becomes a small trie lookup over the user's sequences, so a map may bind `gd` without jotter knowing `g` is a prefix. Counts stay digits before the sequence.
- **Scope.** Notebook mode, `V` mode, and the jotter-level keys inside the cell editor (run bindings, split, the completion and signature keys, leave-editor). The builtin editor's vi keys are not remappable: Neovim users get their own mappings through the nvim backend, and the builtin editor stays a fixed vi subset.
- **Validation at startup.** Unknown action, unparsable key, the same sequence bound twice, a sequence that is a prefix of another: a warning in the status line and the default map kept, like a bad config today. Marks (`m`+letter) and the count digits are reserved.
- **Docs.** The keybindings reference shows the defaults and the notation; a short "Key remapping" guide with two or three real maps (arrows, `Space`/`Enter` for running, a leader-style prefix).

## Plan

- [ ] The `Action` table and dispatch through it, no behaviour change; `HELP` and the generated page read it; the drift test covers every action.
- [ ] Key-notation parser and formatter, with tests for modifiers, named keys and sequences.
- [ ] `[keys]` in the config: merge over defaults, validation, conflict detection, warnings.
- [ ] The `?` overlay and the generated keybindings page show the user's map; `jotter --generate` writes the defaults.
- [ ] Docs: the remapping guide and the config reference entry (from the `Config` doc comment, as for every key).

## Things that must keep working

Counts (`3j`, `5G`), marks and `'a`, the jump list, `V` mode and its keys, the `:` command line, completion and signature popups, mouse, the external editor key, and the nvim backend's own mappings inside cells.

## How to tell it worked

A config with `next_cell = "n"` moves with `n`, the `?` overlay shows `n`, and the generated reference still shows `j`. A map binding `g` and `gg` warns and keeps the defaults. `cargo test` has a test per table consumer, and the drift test fails on an action without help text.
