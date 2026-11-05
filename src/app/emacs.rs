//! Application bindings, minibuffer input and incremental search.
use super::*;
use crate::config::Keymap;
use crate::editor::{KillBuffer, alt, ctrl, plain};

#[derive(Default)]
pub(super) struct EmacsUi {
    prefix: Option<char>,
    help_top: usize,
    pub search: Option<ISearch>,
    kill: KillBuffer,
    pub prompt_cursor: Option<usize>,
    pub stdin_cursor: Option<usize>,
}

pub(super) struct ISearch {
    query: String,
    reverse: bool,
    origin: (usize, usize),
    original_cursor: Option<(usize, usize)>,
    position: (usize, usize),
    cell_only: bool,
    found: bool,
    wrapped: bool,
}

fn byte_at(text: &str, offset: usize) -> usize {
    text.char_indices()
        .nth(offset)
        .map_or(text.len(), |(b, _)| b)
}

pub(super) fn insert_input(buf: &mut String, cursor: &mut Option<usize>, text: &str) {
    let pos = cursor
        .unwrap_or_else(|| buf.chars().count())
        .min(buf.chars().count());
    buf.insert_str(byte_at(buf, pos), text);
    *cursor = Some(pos + text.chars().count());
}

// Used by prompts and kernel stdin; never interprets modified letters as text.
fn edit_input(buf: &mut String, cursor: &mut Option<usize>, key: KeyEvent) -> bool {
    let len = buf.chars().count();
    let pos = cursor.unwrap_or(len).min(len);
    let next = if ctrl(key, 'a') || (plain(key) && key.code == KeyCode::Home) {
        0
    } else if ctrl(key, 'e') || (plain(key) && key.code == KeyCode::End) {
        len
    } else if ctrl(key, 'b') || (plain(key) && key.code == KeyCode::Left) {
        pos.saturating_sub(1)
    } else if ctrl(key, 'f') || (plain(key) && key.code == KeyCode::Right) {
        (pos + 1).min(len)
    } else if ctrl(key, 'd') || (plain(key) && key.code == KeyCode::Delete) {
        if pos < len {
            buf.remove(byte_at(buf, pos));
        }
        pos
    } else if plain(key) && key.code == KeyCode::Backspace {
        if pos > 0 {
            buf.remove(byte_at(buf, pos - 1));
        }
        pos.saturating_sub(1)
    } else if plain(key) && matches!(key.code, KeyCode::Char(_)) {
        let KeyCode::Char(ch) = key.code else {
            unreachable!()
        };
        insert_input(buf, cursor, &ch.to_string());
        return true;
    } else {
        return !plain(key);
    };
    *cursor = Some(next);
    true
}

impl App {
    pub(super) fn reset_emacs_input(&mut self) {
        if self.emacs.search.is_some() {
            self.on_key_isearch(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::CONTROL));
        }
        self.emacs.prefix = None;
        self.emacs.prompt_cursor = None;
        self.emacs.stdin_cursor = None;
    }

    pub(super) fn builtin_backend(&self, source: &str) -> Backend {
        Backend::Builtin(if self.keymap == Keymap::Emacs {
            Editor::new_emacs(source, self.emacs.kill.clone())
        } else {
            Editor::new(source)
        })
    }

    pub fn emacs_status(&self) -> Option<String> {
        if self.keymap != Keymap::Emacs {
            return None;
        }
        if let Some(search) = &self.emacs.search {
            return Some(format!(
                "{}I-search{}: {}▏{}",
                if search.found { "" } else { "Failing " },
                if search.reverse { " backward" } else { "" },
                search.query,
                if search.wrapped { " (wrapped)" } else { "" }
            ));
        }
        self.emacs.prefix.map(|p| format!("C-{p} …"))
    }

    pub fn emacs_help_top(&self) -> usize {
        self.emacs.help_top
    }

    pub(super) fn on_key_emacs_help(&mut self, key: KeyEvent) -> bool {
        if self.keymap != Keymap::Emacs || !self.show_help {
            return false;
        }
        let page = (self.body.height as usize / 2).max(1);
        if ctrl(key, 'n') || (plain(key) && key.code == KeyCode::Down) {
            self.emacs.help_top += 1;
        } else if ctrl(key, 'p') || (plain(key) && key.code == KeyCode::Up) {
            self.emacs.help_top = self.emacs.help_top.saturating_sub(1);
        } else if ctrl(key, 'v') || (plain(key) && key.code == KeyCode::PageDown) {
            self.emacs.help_top += page;
        } else if alt(key, KeyCode::Char('v')) || (plain(key) && key.code == KeyCode::PageUp) {
            self.emacs.help_top = self.emacs.help_top.saturating_sub(page);
        } else {
            self.show_help = false;
            self.emacs.help_top = 0;
        }
        true
    }

    pub fn input_cursor(&self, stdin: bool) -> Option<usize> {
        if stdin {
            self.emacs.stdin_cursor
        } else {
            self.emacs.prompt_cursor
        }
    }

    pub fn input_with_cursor(&self, buf: &str, stdin: bool) -> String {
        let pos = if stdin {
            self.emacs.stdin_cursor
        } else {
            self.emacs.prompt_cursor
        };
        let b = byte_at(buf, pos.unwrap_or_else(|| buf.chars().count()));
        format!("{}▏{}", &buf[..b], &buf[b..])
    }

    pub(super) fn emacs_overlay_key(&self, key: KeyEvent) -> KeyEvent {
        if self.keymap == Keymap::Emacs && (self.logs.is_some() || self.pager.is_some()) {
            let code = if ctrl(key, 'g') {
                Some(KeyCode::Esc)
            } else if ctrl(key, 'n') {
                Some(KeyCode::Down)
            } else if ctrl(key, 'p') {
                Some(KeyCode::Up)
            } else if ctrl(key, 'v') {
                Some(KeyCode::PageDown)
            } else if alt(key, KeyCode::Char('v')) {
                Some(KeyCode::PageUp)
            } else {
                None
            };
            if let Some(code) = code {
                return KeyEvent::new(code, KeyModifiers::NONE);
            }
        }
        key
    }

    pub(super) fn on_key_emacs_prompt(&mut self, key: KeyEvent) -> bool {
        if self.keymap != Keymap::Emacs {
            return false;
        }
        if ctrl(key, 'g') {
            self.prompt = None;
            self.overwrite_armed = None;
            self.emacs.prompt_cursor = None;
            return true;
        }
        edit_input(
            &mut self.prompt.as_mut().unwrap().buf,
            &mut self.emacs.prompt_cursor,
            key,
        )
    }

    pub(super) fn on_key_emacs_stdin(&mut self, key: KeyEvent) -> bool {
        if self.keymap != Keymap::Emacs {
            return false;
        }
        // Completion/interrupt of an input request retains the existing behavior.
        if ctrl(key, 'c') || (plain(key) && matches!(key.code, KeyCode::Enter | KeyCode::Esc)) {
            return false;
        }
        if ctrl(key, 'g') {
            return true;
        }
        edit_input(
            &mut self.stdin_req.as_mut().unwrap().buf,
            &mut self.emacs.stdin_cursor,
            key,
        )
    }

    pub(super) fn on_key_emacs(
        &mut self,
        key: KeyEvent,
        rendered: &mut Rendered,
    ) -> Option<Action> {
        if self.keymap != Keymap::Emacs {
            return None;
        }
        if let Some(prefix) = self.emacs.prefix.take() {
            if ctrl(key, 'g') || (plain(key) && key.code == KeyCode::Esc) {
                return Some(Action::None);
            }
            match prefix {
                'x' if ctrl(key, 's') => {
                    if self.is_scratch() {
                        self.emacs_save_as(rendered);
                    } else {
                        self.ex_write(false, rendered);
                    }
                }
                'x' if ctrl(key, 'c') => {
                    self.commit_editor(rendered);
                    self.quit();
                }
                'x' if ctrl(key, 'w') => {
                    self.emacs_save_as(rendered);
                }
                'x' if plain(key) && key.code == KeyCode::Char('u') => {
                    self.emacs_history(false, rendered);
                }
                'x' if ctrl(key, 'e') && self.editor.is_none() => {
                    return Some(Action::ExternalEdit);
                }
                'h' if plain(key) && key.code == KeyCode::Char('?') => {
                    self.show_help = true;
                    self.emacs.help_top = 0;
                }
                _ => self.message = Some(format!("undefined key sequence after C-{prefix}")),
            }
            return Some(Action::None);
        }
        if ctrl(key, 'x') || (ctrl(key, 'h') && self.editor.is_none()) {
            self.emacs.prefix = Some(if ctrl(key, 'x') { 'x' } else { 'h' });
            self.pending = None;
            self.count = None;
            self.completion = None;
            if let Some(Backend::Builtin(e)) = &mut self.editor {
                e.break_emacs_group();
            }
            return Some(Action::None);
        }
        let nvim_editing = matches!(&self.editor, Some(Backend::Nvim(_)));
        if ctrl(key, 'g') && (!nvim_editing || self.completion.is_some()) {
            self.pending = None;
            self.count = None;
            self.visual = None;
            self.completion = None;
            self.signature = None;
            if let Some(Backend::Builtin(e)) = &mut self.editor {
                e.break_emacs_group();
            }
            return Some(Action::None);
        }
        if !nvim_editing && (ctrl(key, 's') || ctrl(key, 'r')) {
            self.start_isearch(ctrl(key, 'r'));
            return Some(Action::None);
        }
        if self.editor.is_some() {
            return None;
        }
        if ctrl(key, 'n') || ctrl(key, 'p') {
            self.pending = None;
            self.count = None;
            return Some(self.on_key_notebook(
                KeyEvent::new(
                    if ctrl(key, 'n') {
                        KeyCode::Down
                    } else {
                        KeyCode::Up
                    },
                    KeyModifiers::NONE,
                ),
                rendered,
            ));
        }
        if alt(key, KeyCode::Char('<')) || alt(key, KeyCode::Char('>')) {
            self.pending = None;
            self.count = None;
            self.mark_jump();
            self.selected = if alt(key, KeyCode::Char('<')) {
                0
            } else {
                self.notebook.cells.len().saturating_sub(1)
            };
            return Some(Action::None);
        }
        if ctrl(key, '/') || ctrl(key, '_') || alt(key, KeyCode::Char('_')) {
            self.emacs_history(alt(key, KeyCode::Char('_')), rendered);
            return Some(Action::None);
        }
        None
    }

    fn emacs_save_as(&mut self, rendered: &mut Rendered) {
        if let Some(src) = self.editor.as_ref().map(|e| e.source()) {
            self.update_cell_source(src, rendered);
        }
        self.overwrite_armed = None;
        self.emacs.prompt_cursor = None;
        self.prompt = Some(Prompt {
            kind: PromptKind::SaveAs,
            buf: self.path.display().to_string(),
        });
    }

    fn emacs_history(&mut self, redo: bool, rendered: &mut Rendered) {
        match &mut self.editor {
            Some(Backend::Builtin(e)) => {
                e.history(redo);
                self.completion = None;
                self.signature = None;
            }
            Some(Backend::Nvim(_)) => {
                self.message = Some("use Neovim's undo / redo inside the cell".into());
            }
            None if redo => self.redo(rendered),
            None => self.undo(rendered),
        }
    }

    fn start_isearch(&mut self, reverse: bool) {
        self.pending = None;
        self.count = None;
        self.completion = None;
        self.signature = None;
        self.visual = None;
        let original_cursor = self.editor.as_ref().map(Backend::cursor);
        let offset = if let Some(Backend::Builtin(e)) = &mut self.editor {
            e.break_emacs_group();
            e.offset()
        } else {
            0
        };
        let origin = (self.selected, offset);
        self.emacs.search = Some(ISearch {
            query: String::new(),
            reverse,
            origin,
            original_cursor,
            position: origin,
            cell_only: self.editor.is_some(),
            found: true,
            wrapped: false,
        });
    }

    pub(super) fn paste_isearch(&mut self, text: &str) {
        self.emacs.search.as_mut().unwrap().query.push_str(text);
        self.update_isearch(false);
    }

    pub(super) fn on_key_isearch(&mut self, key: KeyEvent) {
        if ctrl(key, 'g') || (plain(key) && key.code == KeyCode::Esc) {
            let search = self.emacs.search.take().unwrap();
            self.selected = search.origin.0;
            if let (Some(Backend::Builtin(e)), Some(cursor)) =
                (&mut self.editor, search.original_cursor)
            {
                e.cursor = cursor;
            }
        } else if plain(key) && key.code == KeyCode::Enter {
            let search = self.emacs.search.take().unwrap();
            if !search.query.is_empty() {
                self.search = Some(search.query);
            }
            if self.selected != search.origin.0 {
                let selected = self.selected;
                self.selected = search.origin.0;
                self.mark_jump();
                self.selected = selected;
            }
        } else if ctrl(key, 's') || ctrl(key, 'r') {
            self.emacs.search.as_mut().unwrap().reverse = ctrl(key, 'r');
            self.update_isearch(true);
        } else if plain(key) && key.code == KeyCode::Backspace {
            self.emacs.search.as_mut().unwrap().query.pop();
            self.update_isearch(false);
        } else if plain(key)
            && let KeyCode::Char(c) = key.code
        {
            self.emacs.search.as_mut().unwrap().query.push(c);
            self.update_isearch(false);
        }
    }

    fn update_isearch(&mut self, repeat: bool) {
        let search = self.emacs.search.as_ref().unwrap();
        if search.query.is_empty() {
            self.selected = search.origin.0;
            if let (Some(Backend::Builtin(e)), Some(cursor)) =
                (&mut self.editor, search.original_cursor)
            {
                e.cursor = cursor;
            }
            let search = self.emacs.search.as_mut().unwrap();
            search.position = search.origin;
            search.found = true;
            search.wrapped = false;
            return;
        }
        let mut matches = Vec::new();
        let sources: Vec<_> = if search.cell_only {
            vec![(self.selected, self.editor.as_ref().unwrap().source())]
        } else {
            self.notebook
                .cells
                .iter()
                .enumerate()
                .map(|(i, c)| (i, c.source.clone()))
                .collect()
        };
        for (cell, source) in sources {
            for (offset, (byte, _)) in source.char_indices().enumerate() {
                if source[byte..].starts_with(&search.query) {
                    matches.push((cell, offset));
                }
            }
        }
        let start = if repeat {
            search.position
        } else {
            search.origin
        };
        let found = if search.reverse {
            matches
                .iter()
                .rev()
                .find(|&&p| if repeat { p < start } else { p <= start })
        } else {
            matches
                .iter()
                .find(|&&p| if repeat { p > start } else { p >= start })
        };
        let wrapped = found.is_none();
        let found = found
            .or_else(|| {
                if search.reverse {
                    matches.last()
                } else {
                    matches.first()
                }
            })
            .copied();
        let search = self.emacs.search.as_mut().unwrap();
        search.found = found.is_some();
        search.wrapped = wrapped && found.is_some();
        if let Some(position) = found {
            search.position = position;
            self.selected = position.0;
            if let Some(Backend::Builtin(e)) = &mut self.editor {
                e.set_offset(position.1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::tests::{key, one_cell_app};

    fn c(ch: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(ch), KeyModifiers::CONTROL)
    }
    fn m(ch: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(ch), KeyModifiers::ALT)
    }
    fn code(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }
    fn text(app: &mut App, r: &mut Rendered, text: &str) {
        for ch in text.chars() {
            app.on_key(key(ch), r);
        }
    }
    fn emacs(source: &str) -> (App, Rendered) {
        let (mut app, mut r) = one_cell_app(&format!("emacs-{}", uuid::Uuid::new_v4()));
        app.notebook.cells[0].source = source.into();
        r.rebuild_cell(0, &app.notebook.cells[0]);
        app.keymap = Keymap::Emacs;
        app.disk_mtime = None; // fixture removed its file
        (app, r)
    }

    #[test]
    fn notebook_shortcuts_stay_available_and_modifiers_do_not_fall_through() {
        let (mut app, mut r) = emacs("a");
        for event in [c('d'), c('f'), c('a'), c('k'), m('d'), m('f'), m('q')] {
            app.on_key(event, &mut r);
        }
        assert_eq!(app.notebook.cells.len(), 1);
        assert_eq!(app.selected, 0);
        assert!(!app.should_quit);
        app.on_key(key('a'), &mut r);
        assert_eq!(app.notebook.cells.len(), 2);
        app.on_key(c('p'), &mut r);
        assert_eq!(app.selected, 0);
        app.on_key(key('j'), &mut r);
        assert_eq!(app.selected, 1);
        app.on_key(m('<'), &mut r);
        assert_eq!(app.selected, 0);
        app.on_key(m('>'), &mut r);
        assert_eq!(app.selected, 1);
        text(&mut app, &mut r, "dd");
        assert_eq!(app.notebook.cells.len(), 1);
        app.on_key(c('/'), &mut r);
        assert_eq!(app.notebook.cells.len(), 2);
        app.on_key(m('_'), &mut r);
        assert_eq!(app.notebook.cells.len(), 1);
        app.on_key(c('x'), &mut r);
        app.on_key(key('u'), &mut r);
        assert_eq!(app.notebook.cells.len(), 2);
        app.on_key(c('g'), &mut r);
        assert!(
            app.editor.is_none(),
            "C-g no longer opens the external editor"
        );
        app.on_key(c('x'), &mut r);
        assert!(matches!(app.on_key(c('e'), &mut r), Action::ExternalEdit));
    }

    #[test]
    fn prefix_cancel_unknown_sequence_and_quit_do_not_execute_plain_commands() {
        let (mut app, mut r) = emacs("a");
        app.on_key(c('x'), &mut r);
        assert!(app.emacs_status().unwrap().contains("C-x"));
        app.on_key(key('a'), &mut r);
        assert_eq!(app.notebook.cells.len(), 1);
        assert!(app.message.as_deref().unwrap().contains("undefined"));
        app.on_key(c('x'), &mut r);
        app.on_key(c('g'), &mut r);
        assert!(app.emacs_status().is_none());
        app.on_key(c('h'), &mut r);
        app.on_key(key('?'), &mut r);
        assert!(app.show_help);
        app.on_key(c('g'), &mut r);
        app.on_key(code(KeyCode::Enter), &mut r);
        text(&mut app, &mut r, "新");
        app.on_key(c('x'), &mut r);
        app.on_key(c('c'), &mut r);
        assert_eq!(app.notebook.cells[0].source, "新a");
        assert!(app.editor.is_none());
        assert_eq!(app.confirm, Some(Confirm::Quit));
        assert!(!app.should_quit);
        app.on_key(c('g'), &mut r);
        assert!(app.confirm.is_none());
    }

    #[test]
    fn save_includes_open_buffer_and_keeps_editing_and_save_as_uses_prompt() {
        let (mut app, mut r) = emacs("old");
        let path =
            std::env::temp_dir().join(format!("jotter-emacs-{}.ipynb", uuid::Uuid::new_v4()));
        app.path = path.clone();
        app.on_key(code(KeyCode::Enter), &mut r);
        assert!(matches!(&app.editor, Some(Backend::Builtin(e)) if e.is_emacs()));
        text(&mut app, &mut r, "新");
        app.on_key(c('x'), &mut r);
        app.on_key(c('s'), &mut r);
        assert_eq!(Notebook::open(&path).unwrap().cells[0].source, "新old");
        assert!(app.editor.is_some());
        assert!(!app.dirty);
        app.on_key(c('x'), &mut r);
        app.on_key(c('w'), &mut r);
        assert_eq!(app.prompt.as_ref().unwrap().kind, PromptKind::SaveAs);
        app.on_key(c('g'), &mut r);
        assert!(app.prompt.is_none());
        assert!(app.editor.is_some());
        app.on_key(code(KeyCode::Esc), &mut r);
        app.on_key(c('x'), &mut r);
        app.on_key(c('c'), &mut r);
        assert!(app.should_quit);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn search_in_cell_is_incremental_reversible_wrapping_and_non_mutating() {
        let (mut app, mut r) = emacs("日本 aa\naa 日本");
        app.on_key(code(KeyCode::Enter), &mut r);
        app.on_key(c('e'), &mut r);
        let origin = app.editor.as_ref().unwrap().cursor();
        app.on_key(c('s'), &mut r);
        text(&mut app, &mut r, "日本");
        assert_eq!(app.editor.as_ref().unwrap().cursor(), (1, 3));
        app.on_key(c('s'), &mut r);
        assert_eq!(app.editor.as_ref().unwrap().cursor(), (0, 0));
        assert!(app.emacs_status().unwrap().contains("wrapped"));
        text(&mut app, &mut r, "z");
        assert!(app.emacs_status().unwrap().contains("Failing"));
        assert_eq!(app.editor.as_ref().unwrap().cursor(), (0, 0));
        app.on_key(code(KeyCode::Backspace), &mut r);
        assert_eq!(app.editor.as_ref().unwrap().cursor(), (1, 3));
        app.on_key(c('r'), &mut r);
        assert_eq!(app.editor.as_ref().unwrap().cursor(), (0, 0));
        app.on_key(c('g'), &mut r);
        assert_eq!(app.editor.as_ref().unwrap().cursor(), origin);
        assert_eq!(app.editor.as_ref().unwrap().source(), "日本 aa\naa 日本");
        app.on_key(c('r'), &mut r);
        app.on_paste("日本".into(), &mut r);
        app.on_key(code(KeyCode::Enter), &mut r);
        assert!(app.emacs.search.is_none());
        assert_eq!(app.editor.as_ref().unwrap().cursor(), (0, 0));
        assert!(app.completion.is_none());
    }

    #[test]
    fn notebook_search_restores_selection_on_cancel_and_updates_jump_on_accept() {
        let (mut app, mut r) = emacs("first 日");
        app.on_key(key('a'), &mut r);
        app.notebook.cells[1].source = "next 日".into();
        app.on_key(c('p'), &mut r);
        app.on_key(c('s'), &mut r);
        text(&mut app, &mut r, "日");
        assert_eq!(app.selected, 0);
        app.on_key(c('s'), &mut r);
        assert_eq!(app.selected, 1);
        app.on_key(c('g'), &mut r);
        assert_eq!(app.selected, 0);
        app.on_key(c('r'), &mut r);
        text(&mut app, &mut r, "next");
        assert_eq!(app.selected, 1);
        app.on_key(code(KeyCode::Enter), &mut r);
        assert_eq!(app.search.as_deref(), Some("next"));
        app.on_key(c('o'), &mut r);
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn prompts_move_edit_and_paste_at_unicode_cursor() {
        let (mut app, mut r) = emacs("");
        app.on_key(key(':'), &mut r);
        text(&mut app, &mut r, "日本ab");
        app.on_key(c('a'), &mut r);
        app.on_key(c('f'), &mut r);
        app.on_key(c('d'), &mut r);
        app.on_paste("語".into(), &mut r);
        assert_eq!(app.prompt.as_ref().unwrap().buf, "日語ab");
        assert_eq!(
            app.input_with_cursor(&app.prompt.as_ref().unwrap().buf, false),
            "日語▏ab"
        );
        app.on_key(c('z'), &mut r);
        assert_eq!(app.prompt.as_ref().unwrap().buf, "日語ab");
        app.on_key(c('g'), &mut r);
        app.on_key(key(':'), &mut r);
        text(&mut app, &mut r, "x");
        assert_eq!(
            app.input_with_cursor(&app.prompt.as_ref().unwrap().buf, false),
            "x▏"
        );
    }

    #[test]
    fn kernel_input_takes_priority_over_search_and_uses_its_own_cursor() {
        let (mut app, mut r) = emacs("a");
        app.on_key(c('s'), &mut r);
        text(&mut app, &mut r, "a");
        app.apply_kernel_event(
            Event::Input {
                request: Box::new(JupyterMessage::new(
                    jupyter_protocol::messaging::InputRequest::default(),
                    None,
                )),
                prompt: "input: ".into(),
                password: true,
            },
            &mut r,
        );
        assert!(app.emacs.search.is_none());
        text(&mut app, &mut r, "日本");
        app.on_key(c('b'), &mut r);
        app.on_paste("語".into(), &mut r);
        assert_eq!(app.stdin_req.as_ref().unwrap().buf, "日語本");
        assert_eq!(app.input_cursor(true), Some(2));
        app.on_key(c('c'), &mut r);
        assert!(app.stdin_req.is_none());
    }

    #[test]
    fn scratch_save_asks_for_filename_and_commits_the_open_buffer() {
        let (mut app, mut r) = emacs("a");
        app.path = PathBuf::new();
        app.on_key(code(KeyCode::Enter), &mut r);
        text(&mut app, &mut r, "new");
        app.on_key(c('x'), &mut r);
        app.on_key(c('s'), &mut r);
        assert_eq!(app.notebook.cells[0].source, "newa");
        assert_eq!(app.prompt.as_ref().unwrap().kind, PromptKind::SaveAs);
        let path = std::env::temp_dir().join(format!(
            "jotter-emacs-scratch-{}.ipynb",
            uuid::Uuid::new_v4()
        ));
        app.on_paste(path.to_str().unwrap().into(), &mut r);
        app.on_key(code(KeyCode::Enter), &mut r);
        assert!(app.prompt.is_none());
        assert!(app.editor.is_some());
        assert_eq!(Notebook::open(&path).unwrap().cells[0].source, "newa");
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn builtin_fallback_respects_keymap_and_cancellation_keeps_editor_open() {
        let (mut app, mut r) = emacs("a");
        app.on_key(code(KeyCode::Enter), &mut r);
        app.nvim_failed(anyhow::anyhow!("disconnected"), &mut r);
        assert!(matches!(&app.editor, Some(Backend::Builtin(e)) if e.is_emacs()));
        app.on_key(c(' '), &mut r);
        app.on_key(c('f'), &mut r);
        assert!(app.editor.as_ref().unwrap().region().is_some());
        app.on_key(c('g'), &mut r);
        assert!(app.editor.is_some());
        assert!(app.editor.as_ref().unwrap().region().is_none());
    }

    #[test]
    fn completion_navigation_precedes_text_motions_and_runs_keep_their_behavior() {
        let (mut app, mut r) = emacs("a");
        app.on_key(code(KeyCode::Enter), &mut r);
        app.on_key(c('e'), &mut r);
        let mut completion = Completion::for_test(0, 1);
        completion.push_for_test("abc", "", "");
        completion.push_for_test("abd", "", "");
        completion.all = completion.items.clone();
        completion.source = "a".into();
        app.completion = Some(completion);
        app.on_key(c('n'), &mut r);
        assert_eq!(app.completion.as_ref().unwrap().sel, 1);
        assert_eq!(app.editor.as_ref().unwrap().cursor(), (0, 1));
        app.on_key(c('p'), &mut r);
        app.on_key(code(KeyCode::Enter), &mut r);
        assert_eq!(app.editor.as_ref().unwrap().source(), "abc");
        app.on_key(c('/'), &mut r);
        assert_eq!(app.editor.as_ref().unwrap().source(), "a");
        text(&mut app, &mut r, "b");
        let mut completion = Completion::for_test(0, 2);
        completion.push_for_test("abc", "word", "");
        completion.all = completion.items.clone();
        completion.source = "ab".into();
        app.completion = Some(completion);
        app.on_key(code(KeyCode::Esc), &mut r);
        assert!(app.editor.is_some(), "Esc dismisses the popup first");
        text(&mut app, &mut r, "c");
        app.on_key(c('/'), &mut r);
        assert_eq!(
            app.editor.as_ref().unwrap().source(),
            "ab",
            "dismissing completion separates undo groups"
        );
        app.kernel_starting = true; // queue execution without starting a kernel
        app.on_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::CONTROL), &mut r);
        assert!(app.editor.is_none());
        assert_eq!(app.deferred.len(), 1);
        app.on_key(key('a'), &mut r);
        app.on_key(c('p'), &mut r);
        app.on_key(code(KeyCode::Enter), &mut r);
        app.on_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::SHIFT), &mut r);
        assert!(app.editor.is_none());
        assert_eq!(app.selected, 1);
    }

    #[test]
    fn cancelling_pagers_and_scrolling_help_never_runs_notebook_commands() {
        let (mut app, mut r) = emacs("a");
        app.pager = Some(Pager {
            title: "docs".into(),
            text: "docs".into(),
            top: 0,
        });
        app.on_key(c('n'), &mut r);
        assert_eq!(app.pager.as_ref().unwrap().top, 1);
        app.on_key(c('g'), &mut r);
        assert!(app.pager.is_none());
        app.logs = Some(0);
        app.on_key(c('g'), &mut r);
        assert!(app.logs.is_none());
        app.on_key(c('h'), &mut r);
        app.on_key(key('?'), &mut r);
        app.on_key(c('n'), &mut r);
        assert!(app.show_help);
        assert_eq!(app.emacs_help_top(), 1);
        app.on_key(c('g'), &mut r);
        assert!(!app.show_help);
        assert_eq!(app.notebook.cells.len(), 1);
    }

    #[test]
    #[ignore = "spawns a real nvim"]
    fn nvim_owns_edit_keys_while_application_prefixes_still_work() {
        let (mut app, mut r) = emacs("original");
        let path =
            std::env::temp_dir().join(format!("jotter-emacs-nvim-{}.ipynb", uuid::Uuid::new_v4()));
        app.path = path.clone();
        app.editor = Some(Backend::Nvim(
            NvimCell::open(NvimSession::spawn(false).unwrap(), "emacs", "original", "").unwrap(),
        ));
        app.on_key(key('i'), &mut r);
        text(&mut app, &mut r, "new");
        app.on_key(code(KeyCode::Esc), &mut r);
        assert_eq!(app.editor.as_ref().unwrap().mode(), ModeKind::Normal);
        app.on_key(key('u'), &mut r);
        assert_eq!(app.editor.as_ref().unwrap().source(), "original");
        app.on_key(c('r'), &mut r);
        assert!(app.emacs.search.is_none());
        assert_eq!(app.editor.as_ref().unwrap().source(), "neworiginal");
        app.on_key(c('g'), &mut r);
        assert!(matches!(app.editor, Some(Backend::Nvim(_))));
        app.on_key(c('x'), &mut r);
        app.on_key(c('s'), &mut r);
        assert_eq!(
            Notebook::open(&path).unwrap().cells[0].source,
            "neworiginal"
        );
        app.on_key(c('x'), &mut r);
        app.on_key(c('c'), &mut r);
        assert!(app.should_quit);
        std::fs::remove_file(path).unwrap();
    }
}
