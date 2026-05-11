pub mod api_keys;
pub mod get_cloud_environments;
pub mod get_cloud_object;
//
// OpenWarp Wave 2-2:再删 6 个 AI query —
// `free_available_models` / `get_feature_model_choices` / `get_request_limit_info`
// / `task_attachments` / `get_ai_conversation_format` / `list_ai_conversations`
// — 唯一消费方 `AIClient impl for ServerApi` 已本地 stub Err。
// `get_conversation_usage` 被 `AuthClient` 直调 → 保留(Wave 3 处理 auth)。
// `get_scheduled_agent_history` 中的 `ScheduledAgentHistory` 类型仍被
// `app/src/ai/agent_sdk/schedule.rs` + `ambient_agents/scheduled.rs` 使用,
// operation 本身不再消费但文件保留。
pub mod get_conversation_usage;
pub mod get_oauth_connect_tx_status;
pub mod get_scheduled_agent_history;
pub mod get_updated_cloud_objects;
pub mod get_user;
pub mod get_workspaces_metadata_for_user;
pub mod list_managed_secrets;
pub mod list_warp_dev_images;
pub mod managed_secret_config;
pub mod task_secrets;
