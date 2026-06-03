use crossterm::event::KeyCode;

use crate::app::state::{AppState, Tab};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppEvent {
    Quit,
    ScrollUp,
    ScrollDown,
    PageUp,
    PageDown,
    SwitchTab(Tab),
    ToggleContributorSort,
    Tick,
}

impl AppEvent {
    pub fn from_key_code(code: KeyCode) -> Option<Self> {
        match code {
            KeyCode::Char('q') | KeyCode::Esc => Some(Self::Quit),
            KeyCode::Up => Some(Self::ScrollUp),
            KeyCode::Down => Some(Self::ScrollDown),
            KeyCode::PageUp => Some(Self::PageUp),
            KeyCode::PageDown => Some(Self::PageDown),
            KeyCode::Char('s') => Some(Self::ToggleContributorSort),
            KeyCode::Char(number) => Tab::from_number(number).map(Self::SwitchTab),
            _ => None,
        }
    }

    pub fn apply(self, state: &mut AppState) {
        match self {
            Self::Quit => state.request_quit(),
            Self::ScrollUp => state.select_previous_row(),
            Self::ScrollDown => state.select_next_row(),
            Self::PageUp => state.page_up(),
            Self::PageDown => state.page_down(),
            Self::SwitchTab(tab) => state.switch_tab(tab),
            Self::ToggleContributorSort => state.toggle_contributor_sort(),
            Self::Tick => {}
        }
    }
}
