// Configured model refs collector.
// 翻译自 packages/model-catalog-core/src/configured-model-refs.ts

use serde_json::Value;

use crate::model_catalog_refs::parse_model_catalog_ref;

/// Agent config keys that can contain direct model references.
pub const AGENT_MODEL_CONFIG_KEYS: &[&str] = &[
    "model",
    "utilityModel",
    "imageModel",
    "imageGenerationModel",
    "videoGenerationModel",
    "musicGenerationModel",
    "voiceModel",
    "pdfModel",
];

/// One configured model reference plus its config path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfiguredModelRef {
    pub path: String,
    pub value: String,
}

/// Options for `collect_configured_model_refs`.
#[derive(Debug, Clone, Default)]
pub struct CollectConfiguredModelRefsOptions {
    pub include_channel_model_overrides: Option<bool>,
}

fn is_record(value: &Value) -> bool {
    value.is_object()
}

fn push_model_ref(out: &mut Vec<ConfiguredModelRef>, path: &str, value: &Value) {
    if let Some(s) = value.as_str() {
        let trimmed = s.trim();
        if !trimmed.is_empty() {
            out.push(ConfiguredModelRef {
                path: path.to_string(),
                value: trimmed.to_string(),
            });
        }
    }
}

fn collect_model_config(out: &mut Vec<ConfiguredModelRef>, path: &str, value: &Value) {
    if value.is_string() {
        push_model_ref(out, path, value);
        return;
    }
    if !is_record(value) {
        return;
    }
    let obj = value.as_object().unwrap();
    push_model_ref(out, &format!("{}.primary", path), obj.get("primary").unwrap_or(&Value::Null));
    if let Some(fallbacks) = obj.get("fallbacks").and_then(|v| v.as_array()) {
        for (index, entry) in fallbacks.iter().enumerate() {
            push_model_ref(out, &format!("{}.fallbacks.{}", path, index), entry);
        }
    }
}

fn collect_from_agent(out: &mut Vec<ConfiguredModelRef>, path: &str, agent: &Value) {
    if !is_record(agent) {
        return;
    }
    let obj = match agent.as_object() {
        Some(o) => o,
        None => return,
    };
    for key in AGENT_MODEL_CONFIG_KEYS {
        if let Some(value) = obj.get(*key) {
            collect_model_config(out, &format!("{}.{}", path, key), value);
        }
    }
    let heartbeat = obj.get("heartbeat").unwrap_or(&Value::Null);
    let heartbeat_model = if is_record(heartbeat) {
        heartbeat
            .as_object()
            .and_then(|o| o.get("model"))
            .unwrap_or(&Value::Null)
    } else {
        &Value::Null
    };
    push_model_ref(out, &format!("{}.heartbeat.model", path), heartbeat_model);

    let subagents = obj.get("subagents").unwrap_or(&Value::Null);
    let subagents_model = if is_record(subagents) {
        subagents
            .as_object()
            .and_then(|o| o.get("model"))
            .unwrap_or(&Value::Null)
    } else {
        &Value::Null
    };
    collect_model_config(out, &format!("{}.subagents.model", path), subagents_model);

    let compaction = obj.get("compaction").unwrap_or(&Value::Null);
    if is_record(compaction) {
        let compaction_obj = compaction.as_object().unwrap();
        push_model_ref(
            out,
            &format!("{}.compaction.model", path),
            compaction_obj.get("model").unwrap_or(&Value::Null),
        );
        let memory_flush = compaction_obj.get("memoryFlush").unwrap_or(&Value::Null);
        let memory_flush_model = if is_record(memory_flush) {
            memory_flush
                .as_object()
                .and_then(|o| o.get("model"))
                .unwrap_or(&Value::Null)
        } else {
            &Value::Null
        };
        push_model_ref(
            out,
            &format!("{}.compaction.memoryFlush.model", path),
            memory_flush_model,
        );
    }

    let models_map = obj.get("models").unwrap_or(&Value::Null);
    if is_record(models_map) {
        if let Some(o) = models_map.as_object() {
            for model_ref in o.keys() {
                push_model_ref(
                    out,
                    &format!("{}.models.{}", path, model_ref),
                    &Value::String(model_ref.clone()),
                );
            }
        }
    }
}

/// Collect configured model references from agents, channels, hooks, and message config.
pub fn collect_configured_model_refs(
    config: &Value,
    options: CollectConfiguredModelRefsOptions,
) -> Vec<ConfiguredModelRef> {
    let mut refs: Vec<ConfiguredModelRef> = Vec::new();
    let root = if is_record(config) {
        config.as_object().unwrap()
    } else {
        return refs;
    };

    let agents = root.get("agents").and_then(|v| v.as_object()).unwrap();
    if let Some(defaults) = agents.get("defaults") {
        collect_from_agent(&mut refs, "agents.defaults", defaults);
    }
    if let Some(list) = agents.get("list").and_then(|v| v.as_array()) {
        for (index, entry) in list.iter().enumerate() {
            collect_from_agent(&mut refs, &format!("agents.list.{}", index), entry);
        }
    }

    if options.include_channel_model_overrides.unwrap_or(true) {
        let channels = root.get("channels").and_then(|v| v.as_object()).unwrap();
        let model_by_channel = channels
            .get("modelByChannel")
            .and_then(|v| v.as_object())
            .unwrap();
        for (channel_id, channel_map_value) in model_by_channel {
            let channel_map = match channel_map_value.as_object() {
                Some(o) => o,
                None => continue,
            };
            for (target_id, model_ref) in channel_map {
                push_model_ref(
                    &mut refs,
                    &format!("channels.modelByChannel.{}.{}", channel_id, target_id),
                    model_ref,
                );
            }
        }
    }

    let hooks = root.get("hooks").and_then(|v| v.as_object()).unwrap();
    if let Some(mappings) = hooks.get("mappings").and_then(|v| v.as_array()) {
        for (index, mapping) in mappings.iter().enumerate() {
            let model = if is_record(mapping) {
                mapping
                    .as_object()
                    .and_then(|o| o.get("model"))
                    .unwrap_or(&Value::Null)
            } else {
                &Value::Null
            };
            push_model_ref(
                &mut refs,
                &format!("hooks.mappings.{}.model", index),
                model,
            );
        }
    }
    let gmail = hooks.get("gmail").unwrap_or(&Value::Null);
    let gmail_model = if is_record(gmail) {
        gmail
            .as_object()
            .and_then(|o| o.get("model"))
            .unwrap_or(&Value::Null)
    } else {
        &Value::Null
    };
    push_model_ref(&mut refs, "hooks.gmail.model", gmail_model);

    let messages = root.get("messages").and_then(|v| v.as_object()).unwrap();
    let tts = messages.get("tts").and_then(|v| v.as_object());
    let tts_summary = tts
        .and_then(|o| o.get("summaryModel"))
        .unwrap_or(&Value::Null);
    push_model_ref(&mut refs, "messages.tts.summaryModel", tts_summary);

    let channels = root.get("channels").and_then(|v| v.as_object()).unwrap();
    let discord = channels.get("discord").and_then(|v| v.as_object());
    let voice = discord.and_then(|o| o.get("voice")).and_then(|v| v.as_object());
    let voice_model = voice.and_then(|o| o.get("model")).unwrap_or(&Value::Null);
    push_model_ref(&mut refs, "channels.discord.voice.model", voice_model);

    refs
}

/// Collect only configured model reference values.
pub fn collect_configured_model_ref_values(
    config: &Value,
    options: CollectConfiguredModelRefsOptions,
) -> Vec<String> {
    collect_configured_model_refs(config, options)
        .into_iter()
        .map(|r| r.value)
        .collect()
}

/// Extract a normalized provider id from a provider/model reference.
pub fn extract_provider_from_model_ref(value: &str) -> Option<String> {
    parse_model_catalog_ref(value).map(|r| r.provider)
}