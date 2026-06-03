pub mod event;
pub mod state;

use std::io::{self, Stdout};
use std::time::Duration;

use crossterm::event::{Event, KeyEventKind, poll, read};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::prelude::CrosstermBackend;

use crate::analytics::AnalysisManager;
use crate::app::event::AppEvent;
use crate::app::state::AppState;
use crate::git::GitRepository;
use crate::ui::dashboard::draw_dashboard;
use crate::utils::Result;

const EVENT_POLL_TIMEOUT: Duration = Duration::from_millis(200);

type AppTerminal = Terminal<CrosstermBackend<Stdout>>;

pub fn run() -> Result<()> {
    let repository = GitRepository::open_current_dir()?;
    let snapshot = AnalysisManager::default().analyze(&repository)?;
    let state = AppState::with_repository(
        snapshot.summary,
        snapshot.contributors,
        snapshot.timeline,
        snapshot.hotspots,
        snapshot.health_score,
        snapshot.bus_factor,
    );

    run_tui(state)
}

fn run_tui(mut state: AppState) -> Result<()> {
    let mut terminal = init_terminal()?;
    let _guard = TerminalGuard;

    while !state.should_quit {
        terminal.draw(|frame| draw_dashboard(frame, &state))?;
        poll_event(&mut state)?;
    }

    Ok(())
}

fn init_terminal() -> Result<AppTerminal> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    Ok(Terminal::new(backend)?)
}

fn poll_event(state: &mut AppState) -> Result<()> {
    if !poll(EVENT_POLL_TIMEOUT)? {
        AppEvent::Tick.apply(state);
        return Ok(());
    }

    match read()? {
        Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
            if let Some(app_event) = AppEvent::from_key_code(key_event.code) {
                app_event.apply(state);
            }
        }
        _ => {}
    }

    Ok(())
}

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}
