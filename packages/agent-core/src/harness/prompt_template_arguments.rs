// Harness prompt template arguments.
// 翻译自 packages/agent-core/src/harness/prompt-template-arguments.ts

use std::collections::HashMap;

/// Resolves `{key}` placeholders in a template with the supplied values.
pub fn resolve_prompt_template(template: &str, values: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in values {
        let placeholder = format!("{{{}}}", key);
        result = result.replace(&placeholder, value);
    }
    result
}