// OpenWarp(本地化,Phase 5):telemetry 宏的 no-op 化版本。
// 原 send_telemetry_sync_* 宏向 RudderStack/Warp Inc 后端同步发送 telemetry event,
// OpenWarp 不需要任何外部 telemetry,但保留宏签名以避免修改全代码库中数百处调用点。
// 宏体只对参数进行最小消费(避免 unused_variables warning),不执行任何 I/O。
//
// 配套:`warp_core::telemetry::send_telemetry_from_{ctx,app_ctx}` 仍在 warp_core 中,
// 它们经由 `record_event` no-op 等机制也不会真正外发,见 `server/telemetry/mod.rs`
// 的 P1 no-op 注释。

#[macro_export]
macro_rules! send_telemetry_sync_from_ctx {
    ($event:expr, $ctx:expr) => {{
        // OpenWarp:no-op,仅消费输入以避免 unused_variables warning。
        let _ = &$event;
        let _ = &$ctx;
    }};
}

#[macro_export]
macro_rules! send_telemetry_sync_from_app_ctx {
    ($event:expr, $app_ctx:expr) => {{
        // OpenWarp:no-op,仅消费输入以避免 unused_variables warning。
        let _ = &$event;
        let _ = &$app_ctx;
    }};
}

#[macro_export]
macro_rules! send_telemetry_on_executor {
    ($auth_state:expr, $event:expr, $executor:expr) => {{
        // OpenWarp:no-op,仅消费输入以避免 unused_variables warning。
        let _ = &$auth_state;
        let _ = &$event;
        let _ = &$executor;
    }};
}
