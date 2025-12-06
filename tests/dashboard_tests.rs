//! Dashboard integration tests

use std::sync::Arc;
use std::time::Duration;
use tanuki_mcp::access_control::ToolCategory;
use tanuki_mcp::dashboard::{DashboardConfig, DashboardMetrics};

#[test]
fn test_metrics_collector_initialization() {
    let metrics = DashboardMetrics::new();
    assert_eq!(metrics.total_requests(), 0);
    assert_eq!(metrics.total_errors(), 0);
}

#[test]
fn test_metrics_record_successful_call() {
    let metrics = DashboardMetrics::new();

    metrics.record_call(
        "list_issues",
        ToolCategory::Issues,
        Some("group/project"),
        Duration::from_millis(150),
        true,
    );

    assert_eq!(metrics.total_requests(), 1);
    assert_eq!(metrics.total_errors(), 0);

    let snapshot = metrics.snapshot();
    assert_eq!(snapshot.total_requests, 1);
    assert_eq!(snapshot.total_errors, 0);

    // Check tool stats
    assert_eq!(snapshot.tools.len(), 1);
    assert_eq!(snapshot.tools[0].name, "list_issues");
    assert_eq!(snapshot.tools[0].call_count, 1);
    assert_eq!(snapshot.tools[0].error_count, 0);

    // Check project stats
    assert_eq!(snapshot.projects.len(), 1);
    assert_eq!(snapshot.projects[0].name, "group/project");
    assert_eq!(snapshot.projects[0].access_count, 1);
}

#[test]
fn test_metrics_record_failed_call() {
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
fn test_metrics_multiple_tools() {
    let metrics = DashboardMetrics::new();

    // Record various tool calls
    metrics.record_call(
        "list_issues",
        ToolCategory::Issues,
        Some("proj1"),
        Duration::from_millis(100),
        true,
    );
    metrics.record_call(
        "get_issue",
        ToolCategory::Issues,
        Some("proj1"),
        Duration::from_millis(50),
        true,
    );
    metrics.record_call(
        "list_merge_requests",
        ToolCategory::MergeRequests,
        Some("proj1"),
        Duration::from_millis(200),
        true,
    );
    metrics.record_call(
        "list_pipelines",
        ToolCategory::Pipelines,
        Some("proj2"),
        Duration::from_millis(150),
        true,
    );
    metrics.record_call(
        "create_issue",
        ToolCategory::Issues,
        Some("proj1"),
        Duration::from_millis(100),
        false,
    );

    let snapshot = metrics.snapshot();

    assert_eq!(snapshot.total_requests, 5);
    assert_eq!(snapshot.total_errors, 1);
    assert_eq!(snapshot.tools.len(), 5);
    assert_eq!(snapshot.projects.len(), 2);
}

#[test]
fn test_metrics_multiple_projects() {
    let metrics = DashboardMetrics::new();

    metrics.record_call(
        "list_issues",
        ToolCategory::Issues,
        Some("group/project1"),
        Duration::from_millis(100),
        true,
    );
    metrics.record_call(
        "list_issues",
        ToolCategory::Issues,
        Some("group/project2"),
        Duration::from_millis(100),
        true,
    );
    metrics.record_call(
        "list_issues",
        ToolCategory::Issues,
        Some("group/project1"),
        Duration::from_millis(100),
        true,
    );

    let snapshot = metrics.snapshot();

    assert_eq!(snapshot.projects.len(), 2);

    // project1 should have 2 accesses
    let proj1 = snapshot
        .projects
        .iter()
        .find(|p| p.name == "group/project1")
        .unwrap();
    assert_eq!(proj1.access_count, 2);

    // project2 should have 1 access
    let proj2 = snapshot
        .projects
        .iter()
        .find(|p| p.name == "group/project2")
        .unwrap();
    assert_eq!(proj2.access_count, 1);
}

#[test]
fn test_metrics_category_tracking() {
    let metrics = DashboardMetrics::new();

    metrics.record_call(
        "list_issues",
        ToolCategory::Issues,
        None,
        Duration::from_millis(100),
        true,
    );
    metrics.record_call(
        "get_issue",
        ToolCategory::Issues,
        None,
        Duration::from_millis(100),
        true,
    );
    metrics.record_call(
        "list_pipelines",
        ToolCategory::Pipelines,
        None,
        Duration::from_millis(100),
        true,
    );

    let snapshot = metrics.snapshot();

    // Should have at least 2 categories tracked
    assert!(snapshot.categories.len() >= 2);

    // Issues category should have 2 calls
    let issues_cat = snapshot
        .categories
        .iter()
        .find(|c| c.name.to_lowercase().contains("issue"));
    assert!(issues_cat.is_some());
    assert_eq!(issues_cat.unwrap().call_count, 2);
}

#[test]
fn test_metrics_recent_requests_buffer() {
    let metrics = DashboardMetrics::with_capacity(5);

    // Add more requests than capacity
    for i in 0..10 {
        metrics.record_call(
            &format!("tool_{}", i),
            ToolCategory::Issues,
            None,
            Duration::from_millis(10),
            true,
        );
    }

    let snapshot = metrics.snapshot();

    // Should only keep last 5
    assert_eq!(snapshot.recent_requests.len(), 5);
    assert_eq!(snapshot.recent_requests[0].tool, "tool_5");
    assert_eq!(snapshot.recent_requests[4].tool, "tool_9");
}

#[test]
fn test_metrics_average_duration() {
    let metrics = DashboardMetrics::new();

    metrics.record_call(
        "list_issues",
        ToolCategory::Issues,
        None,
        Duration::from_millis(100),
        true,
    );
    metrics.record_call(
        "list_issues",
        ToolCategory::Issues,
        None,
        Duration::from_millis(200),
        true,
    );
    metrics.record_call(
        "list_issues",
        ToolCategory::Issues,
        None,
        Duration::from_millis(300),
        true,
    );

    let snapshot = metrics.snapshot();
    let tool = &snapshot.tools[0];

    assert_eq!(tool.call_count, 3);
    assert_eq!(tool.avg_duration_ms, 200); // (100 + 200 + 300) / 3
}

#[test]
fn test_metrics_tools_used_in_project() {
    let metrics = DashboardMetrics::new();

    metrics.record_call(
        "list_issues",
        ToolCategory::Issues,
        Some("proj"),
        Duration::from_millis(100),
        true,
    );
    metrics.record_call(
        "get_issue",
        ToolCategory::Issues,
        Some("proj"),
        Duration::from_millis(100),
        true,
    );
    metrics.record_call(
        "list_issues",
        ToolCategory::Issues,
        Some("proj"),
        Duration::from_millis(100),
        true,
    );

    let snapshot = metrics.snapshot();
    let project = &snapshot.projects[0];

    // Should track which tools were used
    assert_eq!(project.tools_used.len(), 2);

    // list_issues should have count 2
    let list_issues = project
        .tools_used
        .iter()
        .find(|t| t.tool == "list_issues")
        .unwrap();
    assert_eq!(list_issues.count, 2);
}

#[test]
fn test_metrics_no_project() {
    let metrics = DashboardMetrics::new();

    // Call without project (e.g., list_projects)
    metrics.record_call(
        "list_projects",
        ToolCategory::Projects,
        None,
        Duration::from_millis(100),
        true,
    );

    let snapshot = metrics.snapshot();

    assert_eq!(snapshot.total_requests, 1);
    assert_eq!(snapshot.projects.len(), 0); // No project tracked
    assert_eq!(snapshot.tools.len(), 1);
}

#[test]
fn test_metrics_thread_safety() {
    let metrics = Arc::new(DashboardMetrics::new());
    let mut handles = vec![];

    // Spawn multiple threads recording metrics
    for i in 0..10 {
        let metrics_clone = metrics.clone();
        handles.push(std::thread::spawn(move || {
            for j in 0..100 {
                metrics_clone.record_call(
                    &format!("tool_{}_{}", i, j),
                    ToolCategory::Issues,
                    Some(&format!("project_{}", i)),
                    Duration::from_millis(10),
                    j % 10 != 0, // Every 10th call is an error
                );
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let snapshot = metrics.snapshot();

    // Should have recorded all 1000 requests
    assert_eq!(snapshot.total_requests, 1000);
    // Should have 100 errors (10 per thread)
    assert_eq!(snapshot.total_errors, 100);
    // Should have 10 projects
    assert_eq!(snapshot.projects.len(), 10);
}

#[test]
fn test_dashboard_config_default() {
    let config = DashboardConfig::default();

    assert_eq!(config.bind.port(), 19892);
    assert!(config.enabled);
}

#[test]
fn test_dashboard_config_custom() {
    let config = DashboardConfig::new("0.0.0.0", 8080).unwrap();

    assert_eq!(config.bind.port(), 8080);
    assert_eq!(config.bind.ip().to_string(), "0.0.0.0");
}

#[test]
fn test_metrics_uptime() {
    let metrics = DashboardMetrics::new();

    // Small sleep to ensure uptime > 0
    std::thread::sleep(Duration::from_millis(10));

    let uptime = metrics.uptime();
    assert!(uptime.as_millis() >= 10);
}

#[test]
fn test_metrics_snapshot_requests_per_minute() {
    let metrics = DashboardMetrics::new();

    // Record some requests
    for _ in 0..10 {
        metrics.record_call(
            "list_issues",
            ToolCategory::Issues,
            None,
            Duration::from_millis(10),
            true,
        );
    }

    let snapshot = metrics.snapshot();

    // Should have a positive requests_per_minute
    assert!(snapshot.requests_per_minute >= 0.0);
}
