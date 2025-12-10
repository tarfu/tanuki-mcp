//! Metrics collection for the dashboard
//!
//! Thread-safe metrics collection for tracking tool usage, project access,
//! and request statistics.

use crate::access_control::ToolCategory;
use serde::Serialize;
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
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
    /// Combined metrics data (single lock for all collections)
    data: RwLock<MetricsData>,
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

/// Combined metrics data protected by a single lock
/// This reduces lock acquisitions from 4 to 1 per request
#[derive(Default)]
struct MetricsData {
    tool_stats: HashMap<String, ToolStatsInner>,
    project_stats: HashMap<String, ProjectStatsInner>,
    category_stats: HashMap<ToolCategory, CategoryStatsInner>,
    recent_requests: VecDeque<RequestRecord>,
}

/// Record of a recent request with audit information
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
    /// Request ID for tracing correlation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// Access control decision (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_decision: Option<String>,
    /// Error details (if the request failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_details: Option<String>,
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
            data: RwLock::new(MetricsData {
                tool_stats: HashMap::new(),
                project_stats: HashMap::new(),
                category_stats: HashMap::new(),
                recent_requests: VecDeque::with_capacity(max_recent_requests),
            }),
            max_recent_requests,
        }
    }

    // Helper methods for safe lock access with poison recovery
    // These recover from poisoned locks by logging a warning and continuing with the data

    fn write_data(&self) -> RwLockWriteGuard<'_, MetricsData> {
        self.data.write().unwrap_or_else(|poisoned| {
            tracing::warn!("metrics data lock poisoned, recovering");
            poisoned.into_inner()
        })
    }

    fn read_data(&self) -> RwLockReadGuard<'_, MetricsData> {
        self.data.read().unwrap_or_else(|poisoned| {
            tracing::warn!("metrics data lock poisoned, recovering");
            poisoned.into_inner()
        })
    }

    /// Record a tool call with optional audit information
    pub fn record_call(
        &self,
        tool_name: &str,
        category: ToolCategory,
        project: Option<&str>,
        duration: Duration,
        success: bool,
    ) {
        self.record_call_with_audit(
            tool_name, category, project, duration, success, None, None, None,
        );
    }

    /// Record a tool call with full audit information
    #[allow(clippy::too_many_arguments)]
    pub fn record_call_with_audit(
        &self,
        tool_name: &str,
        category: ToolCategory,
        project: Option<&str>,
        duration: Duration,
        success: bool,
        request_id: Option<&str>,
        access_decision: Option<&str>,
        error_details: Option<&str>,
    ) {
        let duration_ms = duration.as_millis() as u64;
        let now = SystemTime::now();
        let timestamp = now
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        // Update total counters (atomic, outside lock)
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        if !success {
            self.total_errors.fetch_add(1, Ordering::Relaxed);
        }

        // Update all metrics with a single lock acquisition
        let mut data = self.write_data();

        // Update tool stats - avoid String allocation if key exists
        if let Some(entry) = data.tool_stats.get_mut(tool_name) {
            entry.call_count += 1;
            if !success {
                entry.error_count += 1;
            }
            entry.total_duration_ms += duration_ms;
            entry.last_called = Some(now);
        } else {
            data.tool_stats.insert(
                tool_name.to_string(),
                ToolStatsInner {
                    call_count: 1,
                    error_count: if success { 0 } else { 1 },
                    total_duration_ms: duration_ms,
                    last_called: Some(now),
                },
            );
        }

        // Update category stats
        let cat_entry = data.category_stats.entry(category).or_default();
        cat_entry.call_count += 1;
        if !success {
            cat_entry.error_count += 1;
        }

        // Update project stats if applicable - avoid String allocations when keys exist
        if let Some(proj) = project {
            if let Some(proj_entry) = data.project_stats.get_mut(proj) {
                proj_entry.access_count += 1;
                if let Some(count) = proj_entry.tools_used.get_mut(tool_name) {
                    *count += 1;
                } else {
                    proj_entry.tools_used.insert(tool_name.to_string(), 1);
                }
                proj_entry.last_accessed = Some(now);
            } else {
                let mut tools_used = HashMap::new();
                tools_used.insert(tool_name.to_string(), 1);
                data.project_stats.insert(
                    proj.to_string(),
                    ProjectStatsInner {
                        access_count: 1,
                        tools_used,
                        last_accessed: Some(now),
                    },
                );
            }
        }

        // Record recent request
        if data.recent_requests.len() >= self.max_recent_requests {
            data.recent_requests.pop_front(); // O(1) instead of O(n)
        }
        data.recent_requests.push_back(RequestRecord {
            tool: tool_name.to_string(),
            project: project.map(String::from),
            success,
            duration_ms,
            timestamp,
            request_id: request_id.map(String::from),
            access_decision: access_decision.map(String::from),
            error_details: error_details.map(String::from),
        });
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

        // Read all data with a single lock acquisition
        let data = self.read_data();

        // Get tool stats
        let mut tools: Vec<ToolStats> = data
            .tool_stats
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
        tools.sort_unstable_by(|a, b| b.call_count.cmp(&a.call_count));

        // Get project stats
        let mut projects: Vec<ProjectStats> = data
            .project_stats
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
                tools_used.sort_unstable_by(|a, b| b.count.cmp(&a.count));

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
        projects.sort_unstable_by(|a, b| b.access_count.cmp(&a.access_count));

        // Get category stats
        let mut categories: Vec<CategoryStats> = data
            .category_stats
            .iter()
            .map(|(cat, s)| CategoryStats {
                name: format!("{}", cat),
                call_count: s.call_count,
                error_count: s.error_count,
            })
            .collect();
        categories.sort_unstable_by(|a, b| b.call_count.cmp(&a.call_count));

        // Get recent requests (convert VecDeque to Vec)
        let recent_requests: Vec<_> = data.recent_requests.iter().cloned().collect();

        // Release the lock before computing start_time
        drop(data);

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
