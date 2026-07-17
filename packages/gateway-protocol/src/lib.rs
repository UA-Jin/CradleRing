//! gateway-protocol crate
//! 翻译自 packages/gateway-protocol/src/index.ts (barrel)
//!
//! 暂不译整个 index.ts (2455 行), 优先关注于:
//!   - 顶层 modules: client_info / connect_error_details / frames / version
//!   - 辅助模块: secret_ref_contract / clawhub_trust_error_details /
//!                frame_guards / startup_unavailable / validation_errors
//!   - schema 桶: schema::*

pub mod clawhub_trust_error_details;
pub mod client_info;
pub mod connect_error_details;
pub mod frame_guards;
pub mod frames;
pub mod schema;
pub mod secret_ref_contract;
pub mod startup_unavailable;
pub mod validation_errors;
pub mod version;
