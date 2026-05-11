// OpenWarp(本地化,Phase 5):`WorkspaceClient` trait 是云端 billing / workspace 设置 RPC,
// 本地化场景下无 billing / workspace 概念。trait 定义保留作为测试 mock,所有 impl 改 stub
// 返回 Err(无云端时返回错误等价于"功能不可用"),实际调用方在 UserWorkspaces 已 no-op。

use super::ServerApi;
use crate::workspaces::user_workspaces::WorkspacesMetadataResponse;
use crate::workspaces::workspace::AiOverages;
use anyhow::{anyhow, Result};
use async_trait::async_trait;

use crate::server::ids::ServerId;

#[cfg(test)]
use mockall::{automock, predicate::*};

#[cfg_attr(test, automock)]
#[cfg_attr(not(target_family = "wasm"), async_trait)]
#[cfg_attr(target_family = "wasm", async_trait(?Send))]
pub trait WorkspaceClient: 'static + Send + Sync {
    async fn generate_stripe_billing_portal_link(&self, team_uid: ServerId) -> Result<String>;

    async fn update_usage_based_pricing_settings(
        &self,
        team_uid: ServerId,
        usage_based_pricing_enabled: bool,
        max_monthly_spend_cents: Option<u32>,
    ) -> Result<WorkspacesMetadataResponse>;

    async fn refresh_ai_overages(&self) -> Result<AiOverages>;

    async fn purchase_addon_credits(
        &self,
        team_uid: ServerId,
        credits: i32,
    ) -> Result<WorkspacesMetadataResponse>;

    async fn update_addon_credits_settings(
        &self,
        team_uid: ServerId,
        auto_reload_enabled: Option<bool>,
        max_monthly_spend_cents: Option<i32>,
        selected_auto_reload_credit_denomination: Option<i32>,
    ) -> Result<WorkspacesMetadataResponse>;
}

#[cfg_attr(not(target_family = "wasm"), async_trait)]
#[cfg_attr(target_family = "wasm", async_trait(?Send))]
impl WorkspaceClient for ServerApi {
    async fn generate_stripe_billing_portal_link(&self, _team_uid: ServerId) -> Result<String> {
        Err(anyhow!("Stripe billing portal disabled in OpenWarp"))
    }

    async fn update_usage_based_pricing_settings(
        &self,
        _team_uid: ServerId,
        _usage_based_pricing_enabled: bool,
        _max_monthly_spend_cents: Option<u32>,
    ) -> Result<WorkspacesMetadataResponse> {
        Err(anyhow!("Usage-based pricing disabled in OpenWarp"))
    }

    async fn refresh_ai_overages(&self) -> Result<AiOverages> {
        Err(anyhow!("AI overages disabled in OpenWarp"))
    }

    async fn purchase_addon_credits(
        &self,
        _team_uid: ServerId,
        _credits: i32,
    ) -> Result<WorkspacesMetadataResponse> {
        Err(anyhow!("Add-on credits disabled in OpenWarp"))
    }

    async fn update_addon_credits_settings(
        &self,
        _team_uid: ServerId,
        _auto_reload_enabled: Option<bool>,
        _max_monthly_spend_cents: Option<i32>,
        _selected_auto_reload_credit_denomination: Option<i32>,
    ) -> Result<WorkspacesMetadataResponse> {
        Err(anyhow!("Add-on credits settings disabled in OpenWarp"))
    }
}
