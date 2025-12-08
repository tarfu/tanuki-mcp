//! Tool registry integration tests

use tanuki_mcp::tools::definitions;

#[test]
fn test_all_tools_registered() {
    let mut registry = tanuki_mcp::tools::ToolRegistry::new();
    definitions::register_all_tools(&mut registry);

    // We should have 109 tools registered
    assert!(
        registry.len() >= 100,
        "Expected at least 100 tools, got {}",
        registry.len()
    );
}

#[test]
fn test_tool_categories_complete() {
    let mut registry = tanuki_mcp::tools::ToolRegistry::new();
    definitions::register_all_tools(&mut registry);

    // Check that we have tools from all major categories
    let tool_names: Vec<&str> = registry.tools().map(|t| t.name).collect();

    // Issues
    assert!(
        tool_names
            .iter()
            .any(|n| n.starts_with("list_issues") || n.starts_with("get_issue"))
    );
    assert!(tool_names.iter().any(|n| n.starts_with("create_issue")));

    // Merge Requests
    assert!(tool_names.iter().any(|n| n.contains("merge_request")));

    // Pipelines
    assert!(tool_names.iter().any(|n| n.contains("pipeline")));

    // Repository
    assert!(
        tool_names
            .iter()
            .any(|n| n.contains("file") || n.contains("tree"))
    );

    // Projects
    assert!(tool_names.iter().any(|n| n.contains("project")));

    // Labels
    assert!(tool_names.iter().any(|n| n.contains("label")));

    // Wiki
    assert!(tool_names.iter().any(|n| n.contains("wiki")));

    // Milestones
    assert!(tool_names.iter().any(|n| n.contains("milestone")));
}

#[test]
fn test_tool_names_unique() {
    let mut registry = tanuki_mcp::tools::ToolRegistry::new();
    definitions::register_all_tools(&mut registry);

    let names: Vec<&str> = registry.tools().map(|t| t.name).collect();

    // Check for duplicates
    let mut seen = std::collections::HashSet::new();
    for name in &names {
        assert!(seen.insert(*name), "Duplicate tool name: {}", name);
    }
}

#[test]
fn test_tool_schemas_valid() {
    let mut registry = tanuki_mcp::tools::ToolRegistry::new();
    definitions::register_all_tools(&mut registry);

    for tool in registry.tools() {
        // Each tool should have a non-empty name
        assert!(!tool.name.is_empty(), "Tool has empty name");

        // Each tool should have a description
        assert!(
            !tool.description.is_empty(),
            "Tool {} has empty description",
            tool.name
        );

        // Each tool's schema should be a valid JSON object schema
        // In schemars 1.0, Schema wraps a serde_json::Value
        let schema_value =
            serde_json::to_value(&tool.input_schema).expect("Schema should serialize to JSON");
        assert!(
            schema_value.is_object(),
            "Tool {} schema should be a JSON object",
            tool.name
        );

        // Check that schema has type "object" or has properties (valid for tool input)
        let schema_obj = schema_value.as_object().unwrap();
        let is_object_type = schema_obj
            .get("type")
            .and_then(|v| v.as_str())
            .map(|t| t == "object")
            .unwrap_or(false);
        let has_properties = schema_obj.contains_key("properties");

        assert!(
            is_object_type || has_properties,
            "Tool {} has invalid schema structure (not an object type)",
            tool.name
        );
    }
}
