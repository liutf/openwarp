// OpenWarp Wave 4-3:firebase crate 仅保留 `FirebaseError` 一个类型。
//
// 历史职责:封装 Google Identity Platform / Firebase Auth REST 响应 schema —
// `AccountInfo` / `GetAccountInfoResponse` / `FetchAccessTokenResponse` 等
// 直接对应 `/v1/accounts/lookup` 与 `/v1/token` 的成功/失败 payload。
// OpenWarp 已物理删除整条云端身份链路(`AuthClient impl for ServerApi` /
// `app/src/auth/` 下所有 RPC 入口),这些 schema 类型不再有任何 0-内部
// 引用,本次一并物理删:
//   - AccountInfo / GetAccountInfoResponse / GetAccountInfoResponsePayload
//   - FetchAccessTokenResponse / ProviderUserInfo
//
// 保留 `FirebaseError`:它仍作为 [`crate::auth::UserAuthenticationError::
// DeniedAccessToken`] / `UserAccountDisabled` 两个 variant 的 payload type
// 出现在 `app/src/auth/mod.rs`(facade,自身不再构造,但 `root_view.rs` 仍
// `match` 这两个 variant 以保持 UI 代码 0 改动)。W4-S2 删除 `sync_queue` 后
// 该类型在 OpenWarp 范围内不再有任何构造点,Wave 5 可整 crate 一并物理删,届时
// 把 struct 内联到 `auth/mod.rs` 即可。

use serde::{Deserialize, Serialize};

/// Format for error response payloads for Google APIs.
///
/// This error format is standardized across 'v1' Google APIs; its used for both
/// POST /v1/accounts/lookup and POST /v1/token requests.
///
/// 仅作为 `UserAuthenticationError::DeniedAccessToken` / `UserAccountDisabled` 的
/// payload 占位保留;OpenWarp 不再构造该错误。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirebaseError {
    pub code: i32,
    pub message: String,
}

impl std::error::Error for FirebaseError {}

impl std::fmt::Display for FirebaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Firebase request failed with status {} and message: {}",
            self.code, self.message
        )
    }
}
