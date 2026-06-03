use crate::analytics::{BusFactorReport, FileHotspot, HealthScore};
use crate::models::{ContributorStats, RepositorySummary, TimelineEntry};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Tab {
    #[default]
    Overview,
    Contributors,
    Timeline,
    Hotspots,
    Health,
}

impl Tab {
    pub const ALL: [Self; 5] = [
        Self::Overview,
        Self::Contributors,
        Self::Timeline,
        Self::Hotspots,
        Self::Health,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Contributors => "Contributors",
            Self::Timeline => "Timeline",
            Self::Hotspots => "Hotspots",
            Self::Health => "Health",
        }
    }

    pub fn from_number(number: char) -> Option<Self> {
        match number {
            '1' => Some(Self::Overview),
            '2' => Some(Self::Contributors),
            '3' => Some(Self::Timeline),
            '4' => Some(Self::Hotspots),
            '5' => Some(Self::Health),
            _ => None,
        }
    }
}

pub type ActiveTab = Tab;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ContributorSortMode {
    #[default]
    CommitCount,
    ActiveDays,
}

impl ContributorSortMode {
    pub fn toggle(self) -> Self {
        match self {
            Self::CommitCount => Self::ActiveDays,
            Self::ActiveDays => Self::CommitCount,
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::CommitCount => "Commit Count",
            Self::ActiveDays => "Active Days",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct AppState {
    pub active_tab: Tab,
    pub should_quit: bool,
    pub selected_row: usize,
    pub repository: Option<RepositorySummary>,
    pub contributors: Vec<ContributorStats>,
    pub timeline: Vec<TimelineEntry>,
    pub hotspots: Vec<FileHotspot>,
    pub health_score: Option<HealthScore>,
    pub bus_factor: Option<BusFactorReport>,
    pub contributor_sort_mode: ContributorSortMode,
}

impl AppState {
    pub const PAGE_SCROLL_AMOUNT: usize = 10;

    pub fn with_repository(
        repository: RepositorySummary,
        contributors: Vec<ContributorStats>,
        timeline: Vec<TimelineEntry>,
        hotspots: Vec<FileHotspot>,
        health_score: HealthScore,
        bus_factor: BusFactorReport,
    ) -> Self {
        let mut state = Self {
            repository: Some(repository),
            contributors,
            timeline,
            hotspots,
            health_score: Some(health_score),
            bus_factor: Some(bus_factor),
            ..Self::default()
        };
        state.sort_contributors();
        state
    }

    pub fn with_summary(repository: RepositorySummary) -> Self {
        Self {
            repository: Some(repository),
            ..Self::default()
        }
    }

    pub fn switch_tab(&mut self, tab: Tab) {
        self.active_tab = tab;
        self.selected_row = 0;
    }

    pub fn select_next_row(&mut self) {
        self.selected_row = self.selected_row.saturating_add(1);
    }

    pub fn select_previous_row(&mut self) {
        self.selected_row = self.selected_row.saturating_sub(1);
    }

    pub fn page_down(&mut self) {
        self.selected_row = self.selected_row.saturating_add(Self::PAGE_SCROLL_AMOUNT);
    }

    pub fn page_up(&mut self) {
        self.selected_row = self.selected_row.saturating_sub(Self::PAGE_SCROLL_AMOUNT);
    }

    pub fn request_quit(&mut self) {
        self.should_quit = true;
    }

    pub fn toggle_contributor_sort(&mut self) {
        self.contributor_sort_mode = self.contributor_sort_mode.toggle();
        self.sort_contributors();
        self.selected_row = 0;
    }

    fn sort_contributors(&mut self) {
        match self.contributor_sort_mode {
            ContributorSortMode::CommitCount => self.contributors.sort_by(|left, right| {
                right
                    .commit_count
                    .cmp(&left.commit_count)
                    .then_with(|| left.email.cmp(&right.email))
            }),
            ContributorSortMode::ActiveDays => self.contributors.sort_by(|left, right| {
                right
                    .active_days
                    .cmp(&left.active_days)
                    .then_with(|| left.email.cmp(&right.email))
            }),
        }
    }
}
