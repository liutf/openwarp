pub mod create_managed_secret;
pub mod delete_managed_secret;
pub mod generate_commands;
pub mod generate_metadata_for_command;
// OpenWarp Wave1-2:`give_up_notebook_edit_access` / `grab_notebook_edit_access` /
// `leave_object` / `record_object_action` / `remove_object_guest` 5 个 mutation
// 唯一消费方 (ObjectClient impl) 已本地 stub 为 Err,文件一并物理删除。
//
// OpenWarp Wave 2-1:再删 21 个 cloud-object mutation —
// `add_object_guests` / `bulk_create_objects` / `create_folder` / `create_generic_string_object`
// / `create_notebook` / `create_workflow` / `delete_object` / `empty_trash` / `move_object`
// / `remove_object_link_permissions` / `set_object_link_permissions`
// / `transfer_generic_string_object_owner` / `transfer_notebook_owner` / `transfer_workflow_owner`
// / `trash_object` / `untrash_object` / `update_folder` / `update_generic_string_object`
// / `update_notebook` / `update_object_guests` / `update_workflow` —
// 唯一消费方 `ObjectClient impl for ServerApi` 已 100% 本地化(合成 ServerXxx
// 或 no-op Ok),不再调任何 GraphQL 路径。
//
// OpenWarp Wave 2-2:再删 5 个 AI mutation —
// `confirm_file_artifact_upload` / `create_file_artifact_upload_target`
// / `delete_ai_conversation` / `generate_dialogue` / `request_bonus`
// (`provideNegativeFeedbackResponseForAiConversation`) — 唯一消费方
// `AIClient impl for ServerApi` 已本地 stub Err。
// `generate_commands` / `generate_metadata_for_command` 有复用类型被
// `app/src/ai_assistant` / `app/src/drive/workflows/ai_assist.rs` import,
// 保留 operation 文件;Wave 3 裁掉调用方后可进一步删除。
//
// OpenWarp Wave 3-1:再删 4 个 auth-only mutation —
// `create_anonymous_user` / `expire_api_key` / `generate_api_key` /
// `mint_custom_token` — 唯一消费方
// `AuthClient impl for ServerApi` 已随 server_api/auth.rs 整文件物理删,
// 上层 AuthManager 改为本地 stub,不再发起任何云端身份请求。
// `issue_task_identity_token` 仍被 BYOP AWS / GCP federated credentials 链路
// (`ai/aws_credentials.rs` + `ai/agent_sdk/federate.rs` + `cloud_provider/aws.rs`)
// 消费,保留 operation 文件。
pub mod issue_task_identity_token;
pub mod update_managed_secret;
