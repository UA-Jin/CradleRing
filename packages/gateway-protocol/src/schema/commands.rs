// Gateway Protocol schema: commands.
// 翻译自 packages/gateway-protocol/src/schema/commands.ts
//
// Command catalog protocol schemas.
// Command entries describe native, skill, and plugin commands that clients can
// render or route; limits keep command catalogs bounded for UI and transport.

use serde::{Deserialize, Serialize};

/// Maximum command display/name length accepted in catalog entries.
pub const COMMAND_NAME_MAX_LENGTH: usize = 200;
/// Maximum command description length accepted in catalog entries.
pub const COMMAND_DESCRIPTION_MAX_LENGTH: usize = 2_000;
/// Maximum text aliases advertised for one command.
pub const COMMAND_ALIAS_MAX_ITEMS: usize = 20;
/// Maximum declared arguments advertised for one command.
pub const COMMAND_ARGS_MAX_ITEMS: usize = 20;
/// Maximum argument name length accepted in catalog entries.
pub const COMMAND_ARG_NAME_MAX_LENGTH: usize = 200;
/// Maximum argument description length accepted in catalog entries.
pub const COMMAND_ARG_DESCRIPTION_MAX_LENGTH: usize = 500;
/// Maximum static choices advertised for one argument.
pub const COMMAND_ARG_CHOICES_MAX_ITEMS: usize = 50;
/// Maximum machine-readable choice value length.
pub const COMMAND_CHOICE_VALUE_MAX_LENGTH: usize = 200;
/// Maximum user-facing choice label length.
pub const COMMAND_CHOICE_LABEL_MAX_LENGTH: usize = 200;
/// Maximum commands returned by one catalog response.
pub const COMMAND_LIST_MAX_ITEMS: usize = 500;

/// Source system that contributed a command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CommandSource {
    Native,
    Skill,
    Plugin,
}

/// Surfaces where a command may be invoked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CommandScope {
    Text,
    Native,
    Both,
}

/// Coarse UI grouping for command catalog display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CommandCategory {
    Session,
    Options,
    Status,
    Management,
    Media,
    Tools,
    Docks,
}

/// Static argument choice shown to clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandArgChoice {
    pub value: String,
    pub label: String,
}

impl CommandArgChoice {
    pub fn validate(&self) -> bool {
        self.value.len() <= COMMAND_CHOICE_VALUE_MAX_LENGTH
            && self.label.len() <= COMMAND_CHOICE_LABEL_MAX_LENGTH
    }
}

/// Argument types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CommandArgType {
    String,
    Number,
    Boolean,
}

/// One typed argument advertised for a command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandArg {
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub arg_type: CommandArgType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub choices: Option<Vec<CommandArgChoice>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dynamic: Option<bool>,
}

impl CommandArg {
    pub fn validate(&self) -> bool {
        !self.name.is_empty()
            && self.name.len() <= COMMAND_ARG_NAME_MAX_LENGTH
            && self.description.len() <= COMMAND_ARG_DESCRIPTION_MAX_LENGTH
            && self.choices.as_ref().map_or(true, |c| c.len() <= COMMAND_ARG_CHOICES_MAX_ITEMS)
    }
}

/// One command catalog entry visible to clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandEntry {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "textAliases")]
    pub text_aliases: Option<Vec<String>>,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<CommandCategory>,
    pub source: CommandSource,
    pub scope: CommandScope,
    #[serde(rename = "acceptsArgs")]
    pub accepts_args: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<CommandArg>>,
}

impl CommandEntry {
    pub fn validate(&self) -> bool {
        !self.name.is_empty()
            && self.name.len() <= COMMAND_NAME_MAX_LENGTH
            && self.description.len() <= COMMAND_DESCRIPTION_MAX_LENGTH
            && self
                .text_aliases
                .as_ref()
                .map_or(true, |a| a.len() <= COMMAND_ALIAS_MAX_ITEMS)
            && self
                .args
                .as_ref()
                .map_or(true, |a| a.len() <= COMMAND_ARGS_MAX_ITEMS && a.iter().all(|a| a.validate()))
    }
}

/// Command catalog request filters.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommandsListParams {
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "agentId")]
    pub agent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<CommandScope>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "includeArgs")]
    pub include_args: Option<bool>,
}

/// Bounded command catalog response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandsListResult {
    pub commands: Vec<CommandEntry>,
}

impl CommandsListResult {
    pub fn validate(&self) -> bool {
        self.commands.len() <= COMMAND_LIST_MAX_ITEMS
            && self.commands.iter().all(|c| c.validate())
    }
}

// Wire type aliases (对标 TS `type X = Static<typeof YSchema>`)
pub type CommandEntryType = CommandEntry;
pub type CommandsListParamsType = CommandsListParams;
pub type CommandsListResultType = CommandsListResult;
