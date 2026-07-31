use std::{io, path::Path};

use babomemo::{
    app::App,
    config::Config,
    constants::MEMO_FILE_NAME,
    document::ParseError,
    i18n::I18n,
    storage::{self, LoadError},
};
use crossterm::{
    event::{
        DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture, poll,
        read,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use fluent_bundle::FluentArgs;
use ratatui::{Terminal, backend::CrosstermBackend};

fn main() {
    if let Err(error) = run() {
        eprintln!("babomemo: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let i18n = I18n::load()?;
    let (config, config_error) = match Config::load() {
        Ok(config) => (config, None),
        Err(error) => {
            let mut args = FluentArgs::new();
            args.set("error", error.to_string());
            let message = i18n.format("status-config-error", Some(&args));
            (Config::default(), Some(message))
        }
    };
    let directory = std::env::current_dir().map_err(|error| error.to_string())?;
    let path = storage::memo_path(&directory, MEMO_FILE_NAME);
    let (document, created) =
        storage::load(&path).map_err(|error| localized_load_error(&i18n, &path, error))?;
    if created {
        storage::save(&path, &document)
            .map_err(|error| localized_load_error(&i18n, &path, LoadError::Io(error)))?;
    }

    let mut app = App::new(path, document, config, i18n, config_error);
    run_terminal(&mut app).map_err(|error| error.to_string())
}

fn run_terminal(app: &mut App) -> io::Result<()> {
    let guard = TerminalGuard::enter()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    while !app.should_quit() {
        terminal.draw(|frame| app.render(frame))?;
        if poll(app.poll_timeout())? {
            app.handle_event(read()?);
        }
        app.autosave_if_due();
    }

    terminal.show_cursor()?;
    drop(terminal);
    drop(guard);
    Ok(())
}

struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        if let Err(error) = execute!(
            io::stdout(),
            EnterAlternateScreen,
            EnableMouseCapture,
            EnableBracketedPaste
        ) {
            let _ = disable_raw_mode();
            return Err(error);
        }
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = execute!(
            io::stdout(),
            DisableBracketedPaste,
            DisableMouseCapture,
            LeaveAlternateScreen
        );
        let _ = disable_raw_mode();
    }
}

fn localized_load_error(i18n: &I18n, path: &Path, error: LoadError) -> String {
    let detail = match error {
        LoadError::Io(error) => error.to_string(),
        LoadError::Parse(ParseError::ContentBeforeFirstNote) => {
            i18n.text("error-format-content-before-note")
        }
    };
    let mut args = FluentArgs::new();
    args.set("file", path.to_string_lossy().into_owned());
    args.set("error", detail);
    i18n.format("error-load-prefix", Some(&args))
}
