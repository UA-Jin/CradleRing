//! Azure deployment name resolution.
//! 翻译自 packages/ai/src/providers/azure-deployment-map.ts

use std::collections::HashMap;

use once_cell::sync::Lazy;

use std::sync::RwLock;

static CACHED_LOOKUP: Lazy<RwLock<Option<DeploymentLookup>>> = Lazy::new(|| RwLock::new(None));

struct DeploymentLookup {
    source: Option<String>,
    exact: HashMap<String, String>,
    folded: HashMap<String, String>,
}

/// Parses `AZURE_OPENAI_DEPLOYMENT_MAP`-style `model=deployment` entries.
pub fn parse_azure_deployment_name_map(value: Option<&str>) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let Some(value) = value else {
        return map;
    };
    for entry in value.split(',') {
        let trimmed = entry.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Some(separator) = trimmed.find('=') else {
            continue;
        };
        if separator == 0 {
            continue;
        }
        let model_id = trimmed[..separator].trim().to_string();
        let deployment_name = trimmed[separator + 1..].trim().to_string();
        if model_id.is_empty() || deployment_name.is_empty() {
            continue;
        }
        map.insert(model_id, deployment_name);
    }
    map
}

fn get_deployment_lookup(source: Option<&str>) -> DeploymentLookup {
    {
        let guard = CACHED_LOOKUP.read().expect("azure deployment poisoned");
        if let Some(cached) = guard.as_ref() {
            if cached.source.as_deref() == source {
                return DeploymentLookup {
                    source: cached.source.clone(),
                    exact: cached.exact.clone(),
                    folded: cached.folded.clone(),
                };
            }
        }
    }
    let exact = parse_azure_deployment_name_map(source);
    let mut folded = HashMap::new();
    for (k, v) in &exact {
        folded.insert(k.to_lowercase(), v.clone());
    }
    let lookup = DeploymentLookup {
        source: source.map(|s| s.to_string()),
        exact,
        folded,
    };
    *CACHED_LOOKUP.write().expect("azure deployment poisoned") = Some(DeploymentLookup {
        source: lookup.source.clone(),
        exact: lookup.exact.clone(),
        folded: lookup.folded.clone(),
    });
    lookup
}

/// Resolves the Azure deployment name for a model id, falling back to the model id.
pub fn resolve_azure_deployment_name_from_map(model_id: &str, deployment_map: Option<&str>) -> String {
    let lookup = get_deployment_lookup(deployment_map);
    lookup
        .exact
        .get(model_id)
        .or_else(|| lookup.folded.get(&model_id.to_lowercase()))
        .cloned()
        .unwrap_or_else(|| model_id.to_string())
}