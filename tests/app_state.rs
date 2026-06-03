//! 状态测试：验证 Tab 切换、滚动、退出和贡献者排序状态变化。

use crossterm::event::KeyCode;
use gitinsight_rs::analytics::{BusFactorReport, HealthScore, RiskReport};
use gitinsight_rs::app::event::AppEvent;
use gitinsight_rs::app::state::{AppState, ContributorSortMode, Tab};
use gitinsight_rs::models::{ContributorStats, RepositorySummary};

#[test]
fn numeric_keys_switch_tabs() {
    let mut state = AppState::default();

    AppEvent::from_key_code(KeyCode::Char('3'))
        .expect("3 should map to timeline tab")
        .apply(&mut state);

    assert_eq!(state.active_tab, Tab::Timeline);
    assert_eq!(state.selected_row, 0);

    AppEvent::from_key_code(KeyCode::Char('6'))
        .expect("6 should map to risk tab")
        .apply(&mut state);

    assert_eq!(state.active_tab, Tab::Risk);
    assert_eq!(state.selected_row, 0);
}

#[test]
fn arrow_keys_switch_tabs() {
    let mut state = AppState::default();

    AppEvent::from_key_code(KeyCode::Right)
        .expect("right arrow should switch to next tab")
        .apply(&mut state);
    assert_eq!(state.active_tab, Tab::Contributors);

    AppEvent::from_key_code(KeyCode::Left)
        .expect("left arrow should switch to previous tab")
        .apply(&mut state);
    assert_eq!(state.active_tab, Tab::Overview);

    AppEvent::from_key_code(KeyCode::Left)
        .expect("left arrow should wrap to last tab")
        .apply(&mut state);
    assert_eq!(state.active_tab, Tab::Risk);
}

#[test]
fn scroll_events_update_selected_row_safely() {
    let mut state = AppState::default();

    AppEvent::ScrollUp.apply(&mut state);
    assert_eq!(state.selected_row, 0);

    AppEvent::ScrollDown.apply(&mut state);
    AppEvent::ScrollDown.apply(&mut state);
    assert_eq!(state.selected_row, 2);

    AppEvent::ScrollUp.apply(&mut state);
    assert_eq!(state.selected_row, 1);
}

#[test]
fn page_events_move_by_page_amount() {
    let mut state = AppState::default();

    AppEvent::PageDown.apply(&mut state);
    assert_eq!(state.selected_row, AppState::PAGE_SCROLL_AMOUNT);

    AppEvent::PageUp.apply(&mut state);
    assert_eq!(state.selected_row, 0);
}

#[test]
fn home_and_end_keys_jump_within_active_content() {
    let mut state = AppState {
        contributors: vec![
            ContributorStats::new("Alice", "alice@example.com"),
            ContributorStats::new("Bob", "bob@example.com"),
            ContributorStats::new("Tom", "tom@example.com"),
        ],
        ..AppState::default()
    };
    state.switch_tab(Tab::Contributors);

    AppEvent::from_key_code(KeyCode::End)
        .expect("end should jump to bottom")
        .apply(&mut state);
    assert_eq!(state.selected_row, 2);

    AppEvent::from_key_code(KeyCode::Home)
        .expect("home should jump to top")
        .apply(&mut state);
    assert_eq!(state.selected_row, 0);
}

#[test]
fn quit_key_sets_quit_flag() {
    let mut state = AppState::default();

    AppEvent::from_key_code(KeyCode::Char('q'))
        .expect("q should map to quit")
        .apply(&mut state);

    assert!(state.should_quit);
}

#[test]
fn contributor_sort_toggle_reorders_contributors() {
    let summary = RepositorySummary::new("demo", "D:/repos/demo");
    let mut alice = ContributorStats::new("Alice", "alice@example.com");
    alice.commit_count = 5;
    alice.active_days = 1;
    let mut bob = ContributorStats::new("Bob", "bob@example.com");
    bob.commit_count = 2;
    bob.active_days = 4;

    let mut state = AppState::with_repository(
        summary,
        vec![bob, alice],
        Vec::new(),
        Vec::new(),
        HealthScore::default(),
        BusFactorReport::default(),
        RiskReport::default(),
    );

    assert_eq!(
        state.contributor_sort_mode,
        ContributorSortMode::CommitCount
    );
    assert_eq!(state.contributors[0].name, "Alice");

    AppEvent::ToggleContributorSort.apply(&mut state);

    assert_eq!(state.contributor_sort_mode, ContributorSortMode::ActiveDays);
    assert_eq!(state.contributors[0].name, "Bob");
    assert_eq!(state.selected_row, 0);
}
