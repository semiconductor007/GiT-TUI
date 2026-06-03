//! 应用状态：保存当前页面、滚动位置、分析数据和页面排序方式。

use crate::analytics::{BusFactorReport, FileHotspot, HealthScore, RiskReport};
use crate::models::{ContributorStats, RepositorySummary, TimelineEntry};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Tab {
    #[default]
    Overview,
    Contributors,
    Timeline,
    Hotspots,
    Health,
    Risk,
}

impl Tab {
    pub const ALL: [Self; 6] = [
        Self::Overview,
        Self::Contributors,
        Self::Timeline,
        Self::Hotspots,
        Self::Health,
        Self::Risk,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Self::Overview => "概览",
            Self::Contributors => "贡献者",
            Self::Timeline => "时间线",
            Self::Hotspots => "文件热点",
            Self::Health => "健康度",
            Self::Risk => "风险报告",
        }
    }

    pub fn from_number(number: char) -> Option<Self> {
        match number {
            '1' => Some(Self::Overview),
            '2' => Some(Self::Contributors),
            '3' => Some(Self::Timeline),
            '4' => Some(Self::Hotspots),
            '5' => Some(Self::Health),
            '6' => Some(Self::Risk),
            _ => None,
        }
    }

    pub fn next(self) -> Self {
        let current_index = Self::ALL
            .iter()
            .position(|tab| *tab == self)
            .unwrap_or_default();
        Self::ALL[(current_index + 1) % Self::ALL.len()]
    }

    pub fn previous(self) -> Self {
        let current_index = Self::ALL
            .iter()
            .position(|tab| *tab == self)
            .unwrap_or_default();
        Self::ALL[(current_index + Self::ALL.len() - 1) % Self::ALL.len()]
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
            Self::CommitCount => "提交数量",
            Self::ActiveDays => "活跃天数",
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
    pub risk_report: Option<RiskReport>,
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
        risk_report: RiskReport,
    ) -> Self {
        let mut state = Self {
            repository: Some(repository),
            contributors,
            timeline,
            hotspots,
            health_score: Some(health_score),
            bus_factor: Some(bus_factor),
            risk_report: Some(risk_report),
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

    pub fn select_first_row(&mut self) {
        self.selected_row = 0;
    }

    pub fn select_last_row(&mut self) {
        self.selected_row = self.active_row_count().saturating_sub(1);
    }

    pub fn switch_to_next_tab(&mut self) {
        self.switch_tab(self.active_tab.next());
    }

    pub fn switch_to_previous_tab(&mut self) {
        self.switch_tab(self.active_tab.previous());
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

    fn active_row_count(&self) -> usize {
        match self.active_tab {
            Tab::Overview => self.repository.as_ref().map_or(1, |_| 5),
            Tab::Contributors => self.contributors.len(),
            Tab::Timeline => self.timeline.len(),
            Tab::Hotspots => self.hotspots.len(),
            Tab::Health => {
                usize::from(self.health_score.is_some() && self.bus_factor.is_some()) * 8
            }
            Tab::Risk => self
                .risk_report
                .as_ref()
                .map_or(1, |report| report.reasons.len() + 2),
        }
    }
}
