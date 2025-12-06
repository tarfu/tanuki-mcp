//! Procedural macros for tanuki-mcp
//!
//! This crate provides the `#[gitlab_tool]` attribute macro for defining GitLab MCP tools
//! with minimal boilerplate.

use darling::{FromMeta, ast::NestedMeta};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{DeriveInput, Ident, parse_macro_input};

/// Arguments for the `#[gitlab_tool]` attribute
#[derive(Debug, FromMeta)]
struct GitLabToolArgs {
    /// Tool name (e.g., "create_issue")
    name: String,
    /// Tool description for MCP
    description: String,
    /// Tool category for access control
    category: String,
    /// Operation type: "read", "write", "delete", or "execute"
    operation: String,
    /// Optional: field name containing the project identifier
    #[darling(default)]
    project_field: Option<String>,
}

/// Derive macro for GitLab MCP tools.
///
/// This macro generates:
/// - `Tool` trait implementation (name, description, category, operation_type)
/// - JSON Schema for input arguments via schemars
/// - `AccessControlled` trait implementation
/// - Automatically adds `#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]`
///
/// # Example
///
/// ```ignore
/// #[gitlab_tool(
///     name = "create_issue",
///     description = "Create a new issue in a GitLab project",
///     category = "issues",
///     operation = "write",
///     project_field = "project"
/// )]
/// pub struct CreateIssue {
///     /// Project ID or URL-encoded path
///     pub project: String,
///     /// Issue title
///     pub title: String,
///     /// Issue description (optional)
///     #[serde(default)]
///     pub description: Option<String>,
/// }
///
/// #[async_trait]
/// impl ToolExecutor for CreateIssue {
///     async fn execute(&self, ctx: &ToolContext) -> Result<ToolResult, ToolError> {
///         // Your implementation here
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn gitlab_tool(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr_args = match NestedMeta::parse_meta_list(attr.into()) {
        Ok(v) => v,
        Err(e) => return TokenStream::from(e.to_compile_error()),
    };

    let args = match GitLabToolArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => return TokenStream::from(e.write_errors()),
    };

    let input = parse_macro_input!(item as DeriveInput);
    let expanded = impl_gitlab_tool(&args, &input);

    TokenStream::from(expanded)
}

fn impl_gitlab_tool(args: &GitLabToolArgs, input: &DeriveInput) -> TokenStream2 {
    let struct_name = &input.ident;
    let tool_name = &args.name;
    let description = &args.description;
    let category = &args.category;
    let operation = &args.operation;

    // Convert category string to ToolCategory variant
    let category_variant = match category.as_str() {
        "issues" => quote! { crate::access_control::ToolCategory::Issues },
        "issue_links" => quote! { crate::access_control::ToolCategory::IssueLinks },
        "issue_notes" => quote! { crate::access_control::ToolCategory::IssueNotes },
        "merge_requests" => quote! { crate::access_control::ToolCategory::MergeRequests },
        "mr_discussions" => quote! { crate::access_control::ToolCategory::MrDiscussions },
        "mr_drafts" => quote! { crate::access_control::ToolCategory::MrDrafts },
        "repository" => quote! { crate::access_control::ToolCategory::Repository },
        "branches" => quote! { crate::access_control::ToolCategory::Branches },
        "commits" => quote! { crate::access_control::ToolCategory::Commits },
        "projects" => quote! { crate::access_control::ToolCategory::Projects },
        "namespaces" => quote! { crate::access_control::ToolCategory::Namespaces },
        "labels" => quote! { crate::access_control::ToolCategory::Labels },
        "wiki" => quote! { crate::access_control::ToolCategory::Wiki },
        "pipelines" => quote! { crate::access_control::ToolCategory::Pipelines },
        "milestones" => quote! { crate::access_control::ToolCategory::Milestones },
        "releases" => quote! { crate::access_control::ToolCategory::Releases },
        "users" => quote! { crate::access_control::ToolCategory::Users },
        "groups" => quote! { crate::access_control::ToolCategory::Groups },
        "graphql" => quote! { crate::access_control::ToolCategory::GraphQL },
        "tags" => quote! { crate::access_control::ToolCategory::Tags },
        "search" => quote! { crate::access_control::ToolCategory::Search },
        _ => {
            return syn::Error::new_spanned(input, format!("Unknown category: {}", category))
                .to_compile_error();
        }
    };

    // Convert operation string to OperationType variant
    let operation_variant = match operation.as_str() {
        "read" => quote! { crate::access_control::OperationType::Read },
        "write" => quote! { crate::access_control::OperationType::Write },
        "delete" => quote! { crate::access_control::OperationType::Delete },
        "execute" => quote! { crate::access_control::OperationType::Execute },
        _ => {
            return syn::Error::new_spanned(
                input,
                format!(
                    "Unknown operation: {}. Use: read, write, delete, or execute",
                    operation
                ),
            )
            .to_compile_error();
        }
    };

    // Generate project extraction code
    let project_extraction = if let Some(field_name) = &args.project_field {
        let field_ident = Ident::new(field_name, proc_macro2::Span::call_site());
        quote! {
            fn extract_project(&self) -> Option<String> {
                Some(self.#field_ident.clone())
            }
        }
    } else {
        // Try to auto-detect a "project" field
        quote! {
            fn extract_project(&self) -> Option<String> {
                None
            }
        }
    };

    // Get the visibility, attributes (except our own), and struct body
    let vis = &input.vis;
    let attrs: Vec<_> = input.attrs.iter().collect();
    let generics = &input.generics;

    // Extract fields from the struct
    let fields = match &input.data {
        syn::Data::Struct(data) => match &data.fields {
            syn::Fields::Named(fields) => &fields.named,
            _ => {
                return syn::Error::new_spanned(
                    input,
                    "gitlab_tool only supports structs with named fields",
                )
                .to_compile_error();
            }
        },
        _ => {
            return syn::Error::new_spanned(input, "gitlab_tool only supports structs")
                .to_compile_error();
        }
    };

    quote! {
        #(#attrs)*
        #[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
        #vis struct #struct_name #generics {
            #fields
        }

        impl crate::tools::ToolInfo for #struct_name {
            fn name() -> &'static str {
                #tool_name
            }

            fn description() -> &'static str {
                #description
            }

            fn category() -> crate::access_control::ToolCategory {
                #category_variant
            }

            fn operation_type() -> crate::access_control::OperationType {
                #operation_variant
            }
        }

        impl crate::access_control::AccessControlled for #struct_name {
            fn tool_name(&self) -> &'static str {
                #tool_name
            }

            fn category(&self) -> crate::access_control::ToolCategory {
                #category_variant
            }

            fn operation_type(&self) -> crate::access_control::OperationType {
                #operation_variant
            }

            #project_extraction
        }
    }
}
