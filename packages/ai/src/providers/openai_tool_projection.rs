//! OpenAI tool projection.
//! 翻译自 packages/ai/src/providers/openai-tool-projection.ts
//!
//! Snapshots direct / custom tool descriptors before OpenAI payload
//! construction and produces diagnostic entries for invalid entries.

use normalization_core::is_record;
use serde_json::Value;

/// One projected tool entry.
#[derive(Debug, Clone)]
pub struct OpenAIProjectedTool {
    pub tool_index: i64,
    pub name: String,
    pub description: Option<String>,
    pub parameters: serde_json::Map<String, Value>,
}

/// One diagnostic for an invalid tool.
#[derive(Debug, Clone)]
pub struct OpenAIToolProjectionDiagnostic {
    pub tool_index: i64,
    pub tool_name: Option<String>,
    pub violations: Vec<String>,
}

/// Aggregate result of projecting an OpenAI tool list.
#[derive(Debug, Clone, Default)]
pub struct OpenAIToolProjection {
    pub input_tool_count: usize,
    pub tools: Vec<OpenAIProjectedTool>,
    pub diagnostics: Vec<OpenAIToolProjectionDiagnostic>,
}

/// Raw tool descriptor (caller-side input).
pub trait OpenAIToolDescriptor {
    fn name(&self) -> Option<&str>;
    fn description(&self) -> Option<&str>;
    fn parameters(&self) -> Option<&Value>;
}

fn unreadable_tool_diagnostic(tool_index: i64) -> OpenAIToolProjectionDiagnostic {
    OpenAIToolProjectionDiagnostic {
        tool_index,
        tool_name: None,
        violations: vec![format!("tool[{}] is unreadable", tool_index)],
    }
}

/// Snapshots direct/custom tool descriptors before OpenAI payload construction.
pub fn project_openai_tools(tools: &[&dyn OpenAIToolDescriptor]) -> OpenAIToolProjection {
    let mut projection = OpenAIToolProjection {
        input_tool_count: tools.len(),
        tools: vec![],
        diagnostics: vec![],
    };

    for (tool_index, tool) in tools.iter().enumerate() {
        let tool_index = tool_index as i64;
        let name = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| tool.name())) {
            Ok(name) => name,
            Err(_) => {
                projection
                    .diagnostics
                    .push(unreadable_tool_diagnostic(tool_index));
                continue;
            }
        };
        let Some(name) = name else {
            projection.diagnostics.push(OpenAIToolProjectionDiagnostic {
                tool_index,
                tool_name: None,
                violations: vec![format!("tool[{}].name is empty", tool_index)],
            });
            continue;
        };
        if name.is_empty() {
            projection.diagnostics.push(OpenAIToolProjectionDiagnostic {
                tool_index,
                tool_name: None,
                violations: vec![format!("tool[{}].name is empty", tool_index)],
            });
            continue;
        }
        let description = tool.description().map(|s| s.to_string());
        let params = tool.parameters().cloned().unwrap_or(Value::Object(Default::default()));
        let params = if is_record(&params) {
            params.as_object().unwrap().clone()
        } else {
            serde_json::Map::new()
        };

        projection.tools.push(OpenAIProjectedTool {
            tool_index,
            name: name.to_string(),
            description,
            parameters: params,
        });
    }

    projection
}