//! Metrics collection for the dashboard
//!
//! Thread-safe metrics collection for tracking tool usage, project access,
//! and request statistics.

use crate::access_control::ToolCategory;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime};

/// Dashboard metrics collector
///
/// Thread-safe structure for collecting and reporting metrics about
/// tool usage and project access patterns.
pub struct DashboardMetrics {
    /// Server start time
    start_time: Instant,
    /// Server start time as SystemTime (for display)
    start_system_time: SystemTime,
    /// Total requests processed
    total_requests: AtomicU64,
    /// Total errors
    total_errors: AtomicU64,
    /// Per-tool statistics
    tool_stats: RwLock<HashMap<String, ToolStatsInner>>,
    /// Per-project statistics
    project_stats: RwLock<HashMap<String, ProjectStatsInner>>,
    /// Per-category statistics
    category_stats: RwLock<HashMap<ToolCategory, CategoryStatsInner>>,
    /// Recent requests (circular buffer)
    recent_requests: RwLock<Vec<RequestRecord>>,
    /// Maximum recent requests to keep
    max_recent_requests: usize,
}

/// Internal tool statistics
#[derive(Default)]
struct ToolStatsInner {
    call_count: u64,
    error_count: u64,
    total_duration_ms: u64,
    last_called: Option<SystemTime>,
}

/// Internal project statistics
#[derive(Default)]
struct ProjectStatsInner {
    access_count: u64,
    tools_used: HashMap<String, u64>,
    last_accessed: Option<SystemTime>,
}

/// Internal category statistics
#[derive(Default)]
struct CategoryStatsInner {
    call_count: u64,
    error_count: u64,
}

/// Record of a recent request
#[derive(Clone, Serialize)]
pub struct RequestRecord {
    /// Tool name
    pub tool: String,
    /// Project (if applicable)
    pub project: Option<String>,
    /// Whether the request succeeded
    pub success: bool,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Timestamp
    pub timestamp: u64,
}

/// Serializable tool statistics for API
#[derive(Clone, Serialize)]
pub struct ToolStats {
    pub name: String,
    pub call_count: u64,
    pub error_count: u64,
    pub avg_duration_ms: u64,
    pub last_called: Option<u64>,
}

/// Serializable project statistics for API
#[derive(Clone, Serialize)]
pub struct ProjectStats {
    pub name: String,
    pub access_count: u64,
    pub tools_used: Vec<ToolUsageCount>,
    pub last_accessed: Option<u64>,
}

/// Tool usage count within a project
#[derive(Clone, Serialize)]
pub struct ToolUsageCount {
    pub tool: String,
    pub count: u64,
}

/// Serializable category statistics for API
#[derive(Clone, Serialize)]
pub struct CategoryStats {
    pub name: String,
    pub call_count: u64,
    pub error_count: u64,
}

/// Overall metrics snapshot for API
#[derive(Clone, Serialize)]
pub struct MetricsSnapshot {
    pub uptime_secs: u64,
    pub start_time: u64,
    pub total_requests: u64,
    pub total_errors: u64,
    pub requests_per_minute: f64,
    pub tools: Vec<ToolStats>,
    pub projects: Vec<ProjectStats>,
    pub categories: Vec<CategoryStats>,
    pub recent_requests: Vec<RequestRecord>,
}

impl DashboardMetrics {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self::with_capacity(100)
    }

    /// Create a new metrics collector with specified recent request capacity
    pub fn with_capacity(max_recent_requests: usize) -> Self {
        Self {
            start_time: Instant::now(),
            start_system_time: SystemTime::now(),
            total_requests: AtomicU64::new(0),
            total_errors: AtomicU64::new(0),
            tool_stats: RwLock::new(HashMap::new()),
            project_stats: RwLock::new(HashMap::new()),
            category_stats: RwLock::new(HashMap::new()),
            recent_requests: RwLock::new(Vec::with_capacity(max_recent_requests)),
            max_recent_requests,
        }
    }

    /// Record a tool call
    pub fn record_call(
        &self,
        tool_name: &str,
        category: ToolCategory,
        project: Option<&str>,
        duration: Duration,
        success: bool,
    ) {
        let duration_ms = duration.as_millis() as u64;
        let now = SystemTime::now();
        let timestamp = now
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        // Update total counters
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        if !success {
            self.total_errors.fetch_add(1, Ordering::Relaxed);
        }

        // Update tool stats
        {
            let mut stats = self.tool_stats.write().unwrap();
            let entry = stats.entry(tool_name.to_string()).or_default();
            entry.call_count += 1;
            if !success {
                entry.error_count += 1;
            }
            entry.total_duration_ms += duration_ms;
            entry.last_called = Some(now);
        }

        // Update category stats
        {
            let mut stats = self.category_stats.write().unwrap();
            let entry = stats.entry(category).or_default();
            entry.call_count += 1;
            if !success {
                entry.error_count += 1;
            }
        }

        // Update project stats if applicable
        if let Some(proj) = project {
            let mut stats = self.project_stats.write().unwrap();
            let entry = stats.entry(proj.to_string()).or_default();
            entry.access_count += 1;
            *entry.tools_used.entry(tool_name.to_string()).or_default() += 1;
            entry.last_accessed = Some(now);
        }

        // Record recent request
        {
            let mut recent = self.recent_requests.write().unwrap();
            if recent.len() >= self.max_recent_requests {
                recent.remove(0);
            }
            recent.push(RequestRecord {
                tool: tool_name.to_string(),
                project: project.map(String::from),
                success,
                duration_ms,
                timestamp,
            });
        }
    }

    /// Get a snapshot of all metrics
    pub fn snapshot(&self) -> MetricsSnapshot {
        let uptime = self.start_time.elapsed();
        let uptime_secs = uptime.as_secs();
        let total_requests = self.total_requests.load(Ordering::Relaxed);

        // Calculate requests per minute
        let requests_per_minute = if uptime_secs > 0 {
            (total_requests as f64 / uptime_secs as f64) * 60.0
        } else {
            0.0
        };

        // Get tool stats
        let tools: Vec<ToolStats> = {
            let stats = self.tool_stats.read().unwrap();
            let mut tools: Vec<_> = stats
                .iter()
                .map(|(name, s)| ToolStats {
                    name: name.clone(),
                    call_count: s.call_count,
                    error_count: s.error_count,
                    avg_duration_ms: if s.call_count > 0 {
                        s.total_duration_ms / s.call_count
                    } else {
                        0
                    },
                    last_called: s.last_called.and_then(|t| {
                        t.duration_since(SystemTime::UNIX_EPOCH)
                            .ok()
                            .map(|d| d.as_secs())
                    }),
                })
                .collect();
            tools.sort_by(|a, b| b.call_count.cmp(&a.call_count));
            tools
        };

        // Get project stats
        let projects: Vec<ProjectStats> = {
            let stats = self.project_stats.read().unwrap();
            let mut projects: Vec<_> = stats
                .iter()
                .map(|(name, s)| {
                    let mut tools_used: Vec<_> = s
                        .tools_used
                        .iter()
                        .map(|(tool, count)| ToolUsageCount {
                            tool: tool.clone(),
                            count: *count,
                        })
                        .collect();
                    tools_used.sort_by(|a, b| b.count.cmp(&a.count));

                    ProjectStats {
                        name: name.clone(),
                        access_count: s.access_count,
                        tools_used,
                        last_accessed: s.last_accessed.and_then(|t| {
                            t.duration_since(SystemTime::UNIX_EPOCH)
                                .ok()
                                .map(|d| d.as_secs())
                        }),
                    }
                })
                .collect();
            projects.sort_by(|a, b| b.access_count.cmp(&a.access_count));
            projects
        };

        // Get category stats
        let categories: Vec<CategoryStats> = {
            let stats = self.category_stats.read().unwrap();
            let mut categories: Vec<_> = stats
                .iter()
                .map(|(cat, s)| CategoryStats {
                    name: format!("{}", cat),
                    call_count: s.call_count,
                    error_count: s.error_count,
                })
                .collect();
            categories.sort_by(|a, b| b.call_count.cmp(&a.call_count));
            categories
        };

        // Get recent requests
        let recent_requests = self.recent_requests.read().unwrap().clone();

        let start_time = self
            .start_system_time
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        MetricsSnapshot {
            uptime_secs,
            start_time,
            total_requests,
            total_errors: self.total_errors.load(Ordering::Relaxed),
            requests_per_minute,
            tools,
            projects,
            categories,
            recent_requests,
        }
    }

    /// Get uptime duration
    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Get total request count
    pub fn total_requests(&self) -> u64 {
        self.total_requests.load(Ordering::Relaxed)
    }

    /// Get total error count
    pub fn total_errors(&self) -> u64 {
        self.total_errors.load(Ordering::Relaxed)
    }
}

impl Default for DashboardMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_new() {
        let metrics = DashboardMetrics::new();
        assert_eq!(metrics.total_requests(), 0);
        assert_eq!(metrics.total_errors(), 0);
    }

    #[test]
    fn test_record_successful_call() {
        let metrics = DashboardMetrics::new();
        metrics.record_call(
            "list_issues",
            ToolCategory::Issues,
            Some("group/project"),
            Duration::from_millis(100),
            true,
        );

        assert_eq!(metrics.total_requests(), 1);
        assert_eq!(metrics.total_errors(), 0);

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.total_requests, 1);
        assert_eq!(snapshot.tools.len(), 1);
        assert_eq!(snapshot.tools[0].name, "list_issues");
        assert_eq!(snapshot.projects.len(), 1);
        assert_eq!(snapshot.projects[0].name, "group/project");
    }

    #[test]
    fn test_record_failed_call() {
        let metrics = DashboardMetrics::new();
        metrics.record_call(
            "create_issue",
            ToolCategory::Issues,
            Some("group/project"),
            Duration::from_millis(50),
            false,
        );

        assert_eq!(metrics.total_requests(), 1);
        assert_eq!(metrics.total_errors(), 1);

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.tools[0].error_count, 1);
    }

    #[test]
    fn test_recent_requests_circular_buffer() {
        let metrics = DashboardMetrics::with_capacity(3);

        for i in 0..5 {
            metrics.record_call(
                &format!("tool_{}", i),
                ToolCategory::Issues,
                None,
                Duration::from_millis(10),
                true,
            );
        }

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.recent_requests.len(), 3);
        assert_eq!(snapshot.recent_requests[0].tool, "tool_2");
        assert_eq!(snapshot.recent_requests[2].tool, "tool_4");
    }

    #[test]
    fn test_category_stats() {
        let metrics = DashboardMetrics::new();
        metrics.record_call(
            "list_issues",
            ToolCategory::Issues,
            None,
            Duration::from_millis(10),
            true,
        );
        metrics.record_call(
            "get_issue",
            ToolCategory::Issues,
            None,
            Duration::from_millis(10),
            true,
        );
        metrics.record_call(
            "list_pipelines",
            ToolCategory::Pipelines,
            None,
            Duration::from_millis(10),
            true,
        );

        let snapshot = metrics.snapshot();
        let issues_cat = snapshot
            .categories
            .iter()
            .find(|c| c.name.contains("issues") || c.name.contains("Issues"));
        assert!(issues_cat.is_some());
        assert_eq!(issues_cat.unwrap().call_count, 2);
    }
}
