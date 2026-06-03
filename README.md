<!-- 项目说明：介绍 GitInsight-RS 的功能、架构、运行方式和测试方法。 -->

# GitInsight-RS

GitInsight-RS 是一个基于 Rust 的 TUI 终端应用，用于读取和分析本地 Git
仓库。项目通过 `git2` 读取提交历史，计算仓库指标，并使用 `ratatui` 与
`crossterm` 在终端中展示交互式分析面板。

本项目定位为大学 Rust 课程期末项目。代码优先保证可读性、模块化、可测试性，
并尽量清晰展示 Rust 语言特性，而不是追求过度复杂的工程架构。

## 功能特性

- 仓库概览
  - 仓库名称
  - 提交总数
  - 本地分支数
  - 标签数
  - 贡献者数
- 贡献者分析
  - 提交数量
  - 活跃天数
  - 首次提交时间
  - 最近提交时间
  - 支持按提交数量或活跃天数排序
- 提交时间线
  - 最近提交记录
  - 短提交 ID
  - 作者
  - 提交时间
  - 提交信息
- 文件热点分析
  - 修改最频繁的文件
  - 修改次数
  - 最近修改时间
- Bus Factor 分析
  - 关键贡献者识别
  - 风险等级判断
- 仓库健康度评分
  - 总体分数
  - 活跃度分数
  - 贡献者分布分数
  - Bus Factor 分数
  - 文件热点分数
- 风险报告
  - 综合 Bus Factor、健康度、文件热点和 Ownership
  - 输出可解释风险原因
- TUI 终端交互
  - 页面切换
  - 行滚动
  - 翻页滚动
  - 退出快捷键
- 并发分析
  - 使用 `rayon` 并行加载仓库概览、贡献者、时间线和文件热点分析结果。
  - 基于 Rust 的所有权与类型系统隔离线程任务，每个任务独立打开仓库，避免共享可变状态。

## 技术栈

- Rust 2024
- `git2`
- `ratatui`
- `crossterm`
- `chrono`
- `rayon`
- `anyhow`
- `thiserror`
- `serde`
- `serde_json`
- `tempfile`，用于测试中的临时仓库

## 项目结构

```text
src/
  main.rs
  lib.rs
  app/
    mod.rs
    state.rs
    event.rs
  git/
    mod.rs
    repository.rs
    commit.rs
    analyzer.rs
  analytics/
    mod.rs
    overview.rs
    contributors.rs
    timeline.rs
    hotspot.rs
    bus_factor.rs
    health.rs
    manager.rs
  ui/
    mod.rs
    dashboard.rs
    contributors.rs
    timeline.rs
    hotspot.rs
    health.rs
  models/
    mod.rs
    repository.rs
    commit.rs
    contributor.rs
    timeline.rs
  utils/
    mod.rs
    error.rs
    time.rs
tests/
```

## 架构设计

```text
main.rs
  -> app::run()
      -> GitRepository
      -> AnalysisManager
          -> OverviewAnalyzer
          -> ContributorAnalyzer
          -> TimelineAnalyzer
          -> HotspotAnalyzer
          -> BusFactorAnalyzer
          -> HealthAnalyzer
      -> AppState
      -> ui::dashboard

git
  -> git2 仓库访问
  -> 统一的仓库数据 API

analytics
  -> Analyzer trait 实现
  -> 仓库分析逻辑
  -> 通过 AnalysisManager 进行并发调度

models
  -> 可序列化的数据模型

ui
  -> 只负责 ratatui 渲染

utils
  -> 错误处理与时间工具
```

项目的核心抽象是 `Analyzer` Trait：

```rust
pub trait Analyzer {
    type Output;

    fn analyze(&self, repo: &GitRepository) -> Result<Self::Output>;
}
```

每个分析模块都实现该 Trait，并返回强类型的分析结果。这样可以将数据访问层、
分析逻辑和 TUI 渲染层分离，降低模块之间的耦合。

## 数据流

```text
GitRepository::open_current_dir()
  -> AnalysisManager::analyze()
      -> RepositorySummary
      -> Vec<ContributorStats>
      -> Vec<TimelineEntry>
      -> Vec<FileHotspot>
      -> BusFactorReport
      -> HealthScore
  -> AppState::with_repository()
  -> draw_dashboard()
```

`AnalysisManager` 在线程任务中通过仓库路径重新打开 Git 仓库。这样可以避免在线程
之间直接共享 `git2::Repository`，同时保持对外 API 简洁稳定。

## 安装方法

首先安装或更新 Rust：

```bash
rustup update
```

打开本项目目录后执行构建：

```bash
cargo build
```

## 使用方法

当前版本默认分析进程的当前工作目录。若要分析本项目自身，可以在项目根目录执行：

```bash
cargo run
```

若要分析其他 Git 仓库，可以先构建二进制文件，然后切换到目标仓库目录运行：

```bash
cargo build
cd path/to/another/git/repository
path/to/gitinsight-rs
```

目标目录必须是一个已经存在的 Git 仓库。

## 键盘操作

| 按键 | 功能 |
| --- | --- |
| `1` | 切换到 Overview 页面 |
| `2` | 切换到 Contributors 页面 |
| `3` | 切换到 Timeline 页面 |
| `4` | 切换到 Hotspots 页面 |
| `5` | 切换到 Health 页面 |
| `6` | 切换到 Risk 页面 |
| `Up` | 向上移动选择行 |
| `Down` | 向下移动选择行 |
| `PageUp` | 向上翻页 |
| `PageDown` | 向下翻页 |
| `s` | 切换贡献者排序方式 |
| `q` / `Esc` | 退出程序 |

## TUI 预览

```text
+------------------------------------------------------------+
| GitInsight-RS | Git-TUI                                    |
+------------------------------------------------------------+
| Overview | Contributors | Timeline | Hotspots | Health     |
+------------------------------------------------------------+
| Repository Name: Git-TUI                                   |
| Commits: 53                                                |
| Branches: 4                                                |
| Tags: 1                                                    |
| Contributors: 2                                            |
+------------------------------------------------------------+
| Selected Row: 0                                            |
```

## 测试说明

运行完整校验命令：

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

测试使用 `tempfile` 和 `git2` 动态创建临时 Git 仓库，因此不会依赖本项目真实的
提交历史。

当前测试覆盖范围包括：

- 仓库打开与概览生成
- 贡献者统计、活跃天数、首次和最近提交时间、排序
- 时间线排序、数量限制、空提交信息、短提交 ID
- 文件热点检测与排序
- Bus Factor 分析与风险等级
- 健康度评分范围与评分行为
- 应用状态与 TUI 渲染辅助函数
- AnalysisManager 集成分析

## 错误处理

项目使用统一的应用结果类型：

```rust
pub type Result<T> = std::result::Result<T, AppError>;
```

`AppError` 统一管理 Git 错误、I/O 错误、解析错误和分析错误。代码避免大量使用
`unwrap()`，可恢复错误通过 `Result` 向上传递。

## Rust 课程特性展示

- Ownership 与 Borrowing
  - `GitRepository` 拥有底层 `git2::Repository`。
  - 各分析模块通过 `Analyzer` Trait 借用 `&GitRepository`。
- Trait
  - `Analyzer` 为所有分析模块提供统一接口。
- Struct
  - 使用 `RepositorySummary`、`ContributorStats`、`TimelineEntry`、
    `FileHotspot`、`BusFactorReport`、`HealthScore` 等领域模型。
- Enum
  - 使用 `AppError`、`Tab`、`ContributorSortMode`、`RiskLevel` 等枚举。
- 泛型与关联类型
  - `Analyzer` 通过关联类型表达不同分析器的输出类型。
- Result 错误处理
  - Git 操作和终端操作均返回 `Result<T>`。
- 模块化设计
  - 数据访问、分析逻辑、模型、UI、应用状态和工具函数分层组织。
- 并发编程
  - 使用 `rayon::join` 并行加载相互独立的分析任务。
  - `AnalysisManager` 在线程任务中重新打开仓库，利用 Rust 数据安全与 Rayon 实现并行分析。
- 测试
  - 单元测试和集成测试覆盖核心分析逻辑与 UI 辅助函数。

## 开发注意事项

- 按功能阶段逐步开发。
- 每个阶段完成前运行 `cargo fmt --check`、
  `cargo clippy --all-targets -- -D warnings` 和 `cargo test`。
- 不要替用户将本项目上传到 GitHub 或任何远端仓库。
