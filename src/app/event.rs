//! 事件处理：把键盘输入转换为应用状态变化。

use crossterm::event::KeyCode;

use crate::app::state::{AppState, Tab};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppEvent {
    Quit,
    ScrollUp,
    ScrollDown,
    PageUp,
    PageDown,
    Home,
    End,
    NextTab,
    PreviousTab,
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
            KeyCode::Home => Some(Self::Home),
            KeyCode::End => Some(Self::End),
            KeyCode::Left => Some(Self::PreviousTab),
            KeyCode::Right => Some(Self::NextTab),
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
            Self::Home => state.select_first_row(),
            Self::End => state.select_last_row(),
            Self::NextTab => state.switch_to_next_tab(),
            Self::PreviousTab => state.switch_to_previous_tab(),
            Self::SwitchTab(tab) => state.switch_tab(tab),
            Self::ToggleContributorSort => state.toggle_contributor_sort(),
            Self::Tick => {}
        }
    }
}
