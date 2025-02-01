use std::path::PathBuf;

use nu_ansi_term::{Color, Style};
use reedline::{
    default_emacs_keybindings, ColumnarMenu, DefaultHinter, EditCommand, Emacs, FileBackedHistory,
    KeyCode, KeyModifiers, MenuBuilder, Prompt, Reedline, ReedlineEvent, ReedlineMenu, Signal,
};

use super::completer::{CommandCompleter, FileCompleter};

// TODO: Store the last `HISTORY_CAPACITY` commands in the history file
const HISTORY_CAPACITY: usize = 1024;

pub struct ForgeEditor {
    editor: Reedline,
}

pub enum ReadResult {
    Success(String),
    Continue,
    Exit,
}

impl ForgeEditor {
    fn intialize_bindings() -> reedline::Keybindings {
        let mut keybindings = default_emacs_keybindings();
        // on TAB press shows the completion menu, and if we've exact match it will
        // insert it
        keybindings.add_binding(
            KeyModifiers::NONE,
            KeyCode::Tab,
            ReedlineEvent::UntilFound(vec![
                ReedlineEvent::HistoryHintComplete,
                ReedlineEvent::Menu("completion_menu".to_string()),
                ReedlineEvent::Edit(vec![EditCommand::Complete]),
            ]),
        );

        // on CTRL + k press clears the screen
        keybindings.add_binding(
            KeyModifiers::CONTROL,
            KeyCode::Char('k'),
            ReedlineEvent::ClearScreen,
        );

        // on CTRL + r press searches the history
        keybindings.add_binding(
            KeyModifiers::CONTROL,
            KeyCode::Char('r'),
            ReedlineEvent::SearchHistory,
        );

        // on ALT + Enter press inserts a newline
        keybindings.add_binding(
            KeyModifiers::ALT,
            KeyCode::Enter,
            ReedlineEvent::Edit(vec![EditCommand::InsertNewline]),
        );

        keybindings.add_binding(
            KeyModifiers::NONE,
            KeyCode::Char('/'),
            ReedlineEvent::Menu("command_menu".to_string()),
        );

        keybindings
    }

    pub fn start(cwd: PathBuf) -> Self {
        // Store file history in system config directory
        let history_file = dirs::config_dir()
            .map(|mut path| {
                path.push("forge");
                path.push(".forge_history");
                path
            })
            .unwrap_or_else(|| PathBuf::from(".forge_history"));

        let history = Box::new(
            FileBackedHistory::with_file(HISTORY_CAPACITY, history_file).unwrap_or_default(),
        );
        let completion_menu = Box::new(
            ColumnarMenu::default()
                .with_name("completion_menu")
                .with_marker("")
                .with_text_style(Style::new().bold().fg(Color::White))
                .with_selected_text_style(Style::new().bold().on(Color::White).fg(Color::Black)),
        );

        let commands_menu = Box::new(
            ColumnarMenu::default()
                .with_name("command_menu")
                .with_marker("")
                .with_text_style(Style::new().bold().fg(Color::White))
                .with_selected_text_style(Style::new().bold().on(Color::White).fg(Color::Black)),
        );

        let edit_mode = Box::new(Emacs::new(Self::intialize_bindings()));
        let suggestions_completer = Box::new(FileCompleter::new(cwd));
        let editor = Reedline::create()
            .with_history(history)
            .with_hinter(Box::new(
                DefaultHinter::default().with_style(Style::new().fg(Color::DarkGray)),
            ))
            .with_menu(ReedlineMenu::WithCompleter {
                menu: completion_menu,
                completer: suggestions_completer,
            })
            .with_menu(ReedlineMenu::WithCompleter {
                menu: commands_menu,
                completer: Box::new(CommandCompleter),
            })
            .with_edit_mode(edit_mode)
            .with_quick_completions(true)
            .with_partial_completions(true)
            .with_ansi_colors(true);
        Self { editor }
    }

    pub fn prompt(&mut self, prompt: &dyn Prompt) -> anyhow::Result<ReadResult> {
        let signal = self.editor.read_line(prompt);
        signal.map(Into::into).map_err(|e| anyhow::anyhow!(e))
    }
}

impl From<Signal> for ReadResult {
    fn from(signal: Signal) -> Self {
        match signal {
            Signal::Success(buffer) => ReadResult::Success(buffer),
            Signal::CtrlC => ReadResult::Continue,
            Signal::CtrlD => ReadResult::Exit,
        }
    }
}
