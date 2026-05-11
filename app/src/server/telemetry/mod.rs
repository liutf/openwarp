// OpenWarp(Wave1-S4):闭源 telemetry 完整剥离 — 内部模块与 impl 末梢清除。
//
// 此模块过去负责构造 Rudderstack 批次、序列化为磁盘文件、再以 HTTP 客户端回传 Warp Inc 后端。
// 经 P2 阶段 send 网络层、Wave1-S1..S3 voice/collector/mod_tests 拆除后,本次进一步:
// - 删除 `http_client::Client` 字段(已无任何调用方)
// - 删除 `persist_events_*` / `flush_persisted_events_to_rudder` 等本地落盘 + 回放逻辑
// - 保留 `send_telemetry_event` / `flush_events` / `flush_and_persist_events`
//   / `flush_persisted_events_to_rudder` 接口签名,被 `ServerApi` 透传外部调用,
//   但实现全部 no-op
//
// `TelemetryEvent` 枚举(events.rs 892 行)与 146 文件外部 `import` 保持不动。

pub mod context_provider;
mod events;
mod macros;

pub use events::*;

use crate::auth::UserUid;
use crate::settings::PrivacySettingsSnapshot;
use anyhow::Result;
use serde_json::{json, Value};
use std::path::Path;
use std::sync::OnceLock;

/// OpenWarp(Wave1-S4):原 `context.rs` 已删,但 session_sharing 协议 `InitPayload`
/// 仍含 `TelemetryContext(Value)` 字段(viewer/sharer 之间的协议字段)。这里保留
/// 一个最小 stub:返回一个永远为空 JSON 对象的 `TelemetryContext`,以维持协议兼容。
pub struct TelemetryContext(Value);

impl TelemetryContext {
    pub fn as_value(&self) -> Value {
        self.0.clone()
    }
}

static TELEMETRY_CONTEXT: OnceLock<TelemetryContext> = OnceLock::new();

/// OpenWarp(Wave1-S4):空 telemetry context。原实现会序列化 OS / 用户 agent 等
/// 信息附在 Rudderstack 事件上;现在 Rudderstack 路径已死,仅 session_sharing
/// 协议字段消费,返回空对象即可。
pub fn telemetry_context() -> &'static TelemetryContext {
    TELEMETRY_CONTEXT.get_or_init(|| TelemetryContext(json!({})))
}

pub struct TelemetryApi;

impl Default for TelemetryApi {
    fn default() -> Self {
        Self::new()
    }
}

impl TelemetryApi {
    pub fn new() -> Self {
        Self
    }

    /// 历史:批量上送排队中的 telemetry event 到 Rudderstack。
    /// OpenWarp:no-op,仅 drain 一次内存队列避免堆积,返回 drain 的事件计数。
    pub async fn flush_events(&self, _settings_snapshot: PrivacySettingsSnapshot) -> Result<usize> {
        let events = warpui::telemetry::flush_events();
        Ok(events.len())
    }

    /// 历史:把上一次 quit 时写到磁盘的 event 文件重新发回 Rudderstack。
    /// OpenWarp:no-op,顺手把残留文件清除以避免占用 state 目录。
    pub async fn flush_persisted_events_to_rudder(
        &self,
        path: &Path,
        _settings_snapshot: PrivacySettingsSnapshot,
    ) -> Result<()> {
        if path.exists() {
            if let Err(e) = std::fs::remove_file(path) {
                log::warn!("Failed to remove stale rudder event file {path:?}: {e}");
            }
        }
        Ok(())
    }

    /// 历史:app quit 时把队列尾部 N 条事件写到 state 目录的 JSON,供下次启动回放。
    /// OpenWarp:no-op,仅 drain 一次内存队列。
    pub fn flush_and_persist_events(
        &self,
        _max_event_count: usize,
        _settings_snapshot: PrivacySettingsSnapshot,
    ) -> Result<()> {
        let _ = warpui::telemetry::flush_events();
        Ok(())
    }

    /// 历史:把一条 telemetry event 经 anonymous_id 包装后投递 Rudderstack。
    /// OpenWarp:no-op。仍调用 `warp_core::telemetry::TelemetryEvent::name()` 以
    /// 维持 trait bound 检查(同 `macros.rs`)。
    pub async fn send_telemetry_event(
        &self,
        _user_id: Option<UserUid>,
        _anonymous_id: String,
        event: impl warp_core::telemetry::TelemetryEvent,
        _settings_snapshot: PrivacySettingsSnapshot,
    ) -> Result<()> {
        let _ = event.name();
        Ok(())
    }
}
