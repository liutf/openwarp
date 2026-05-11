mod context;
pub mod context_provider;
mod events;
mod macros;
pub mod rudder_message;
pub mod secret_redaction;

pub use context::telemetry_context;
pub use events::*;

use crate::auth::UserUid;
use crate::server::telemetry_ext::TelemetryExt;
use crate::settings::PrivacySettingsSnapshot;
use crate::ChannelState;
use anyhow::Result;
use futures::FutureExt;
use rudder_message::BatchMessageItem as RudderBatchMessage;
use std::fs::File;
#[cfg(not(target_family = "wasm"))]
use std::fs::OpenOptions;
use std::future::Future;
use std::path::{Path, PathBuf};
use warpui::telemetry::Event;

/// Filename for file where telemetry events are written on app quit.
const RUDDER_TELEMETRY_EVENTS_FILE_NAME: &str = "rudder_telemetry_events.json";

/// Filepath where the Rudder events should be written on app quit.
fn rudder_event_file_path() -> PathBuf {
    warp_core::paths::secure_state_dir()
        .unwrap_or_else(warp_core::paths::state_dir)
        .join(RUDDER_TELEMETRY_EVENTS_FILE_NAME)
}

/// Removes all telemetry events from the app telemetry event queue.
pub fn clear_event_queue() {
    let _ = warpui::telemetry::flush_events();
}

pub struct TelemetryApi {
    pub(super) client: http_client::Client,
}

impl Default for TelemetryApi {
    fn default() -> Self {
        Self::new()
    }
}

impl TelemetryApi {
    pub fn new() -> Self {
        cfg_if::cfg_if! {
            if #[cfg(test)] {
                let client = http_client::Client::new_for_test();
            } else if #[cfg(target_family = "wasm")] {
                let client = http_client::Client::default();
            } else {
                use std::time::Duration;

                let client = http_client::Client::from_client_builder(
                    // We use our own http client directly instead of the Rudderstack SDK's because using
                    // our own client gives us the ability to have universal hooks for pre/post
                    // request/response logic.
                    reqwest::Client::builder()
                        // Don't allow insecure connections; they will be rejected by
                        // the server with a 403 Forbidden.
                        .https_only(true)
                        // Keep idle connections in the pool for up to 55s. AWS
                        // Application Load Balancers will drop idle connections after
                        // 60s and the default pool idle timeout is 90s; a pool idle
                        // timeout longer than the server timeout can lead to errors
                        // upon trying to use an idle connection.
                        .pool_idle_timeout(Duration::from_secs(55))
                        .connect_timeout(Duration::from_secs(10)),
                ).expect("Client should be constructed since we use a compatibility layer to use reqwest::Client");
            }
        }

        Self { client }
    }

    // Batches up telemetry events from the global queue and sends a Message to the Rudderstack API.
    // Returns the number of events that were flushed.
    pub async fn flush_events(&self, _settings_snapshot: PrivacySettingsSnapshot) -> Result<usize> {
        // OpenWarp(P2):闭源遥测剥离 — 不再向 Rudderstack 发送任何事件。
        // 仍消费一次队列以避免事件持续堆积占用内存。
        let events = warpui::telemetry::flush_events();
        Ok(events.len())
    }

    /// Flushes events directly to Rudder that were previously written into a file at `path`
    /// (likely via a call to `write_events_to_disk`).
    pub async fn flush_persisted_events_to_rudder(
        &self,
        path: &Path,
        _settings_snapshot: PrivacySettingsSnapshot,
    ) -> Result<()> {
        // OpenWarp(P2):闭源遥测剥离 — 历史 rudder 事件文件直接丢弃。
        if path.exists() {
            if let Err(e) = std::fs::remove_file(path) {
                log::warn!("Failed to remove stale rudder event file {path:?}: {e}");
            }
        }
        Ok(())
    }

    /// Writes the last `max_event_count` events into disk. This is useful for persisting events
    /// where we can't make a network call to Rudder (such as when the app quits). To flush these
    /// events to Rudder, call `flush_events_to_rudder_from_disk`.
    pub fn flush_and_persist_events(
        &self,
        max_event_count: usize,
        settings_snapshot: PrivacySettingsSnapshot,
    ) -> Result<()> {
        self.flush_and_persist_events_at_path(
            max_event_count,
            settings_snapshot,
            rudder_event_file_path(),
        )
    }

    fn flush_and_persist_events_at_path(
        &self,
        max_event_count: usize,
        settings_snapshot: PrivacySettingsSnapshot,
        path: impl AsRef<Path>,
    ) -> Result<()> {
        if settings_snapshot.should_disable_telemetry() {
            log::info!("Not writing queued events to disk because telemetry is disabled.");
            return Result::Ok(());
        }
        log::info!("Writing queued events to disk because telemetry is enabled.");

        let file = File::create(path)?;

        let events = warpui::telemetry::flush_events();
        if events.len() > max_event_count {
            log::error!("More telemetry events in queue than the limit to persist")
        }

        self.persist_events_at_path(&file, max_event_count, events)?;

        Ok(())
    }

    fn persist_events_at_path(
        &self,
        file: &File,
        max_event_count: usize,
        events: Vec<Event>,
    ) -> Result<()> {
        let rudder_events_to_persist: Vec<RudderBatchMessage> = events
            .into_iter()
            .rev()
            .take(max_event_count)
            .map(TelemetryExt::to_rudder_batch_message)
            .filter_map(|message| (!message.contains_ugc).then_some(message.message))
            .collect();
        serde_json::to_writer(file, &rudder_events_to_persist)?;
        Ok(())
    }

    #[cfg(not(target_family = "wasm"))]
    fn persist_events_to_telemetry_log_file(&self, events: Vec<Event>) -> Result<()> {
        let log_directory = warp_logging::log_directory()?;
        let telemetry_file_path = log_directory.join(&*ChannelState::telemetry_file_name());

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&telemetry_file_path)?;

        self.persist_events_at_path(&file, events.len(), events)
    }

    /// Sends a `TelemetryEvent` to the Rudderstack API.
    pub async fn send_telemetry_event(
        &self,
        user_id: Option<UserUid>,
        anonymous_id: String,
        event: impl warp_core::telemetry::TelemetryEvent,
        settings_snapshot: PrivacySettingsSnapshot,
    ) -> Result<()> {
        let event = warpui::telemetry::create_event(
            user_id.map(|uid| uid.as_string()),
            anonymous_id,
            event.name().into(),
            event.payload(),
            event.contains_ugc(),
            warpui::time::get_current_time(),
        );

        self.send_telemetry_event_internal(event, settings_snapshot)
            .await
    }

    /// Internal implementation for sending telemetry events.
    /// OpenWarp(P2):闭源遥测剥离 — `record_event` 与本路径双 no-op,永不发包。
    /// 历史 Rudder/Sandbox 守护与 send_batch_messages_to_rudder/send_rudder_request
    /// 死代码已在 P2 物理清除。
    fn send_telemetry_event_internal(
        &self,
        _event: Event,
        _settings_snapshot: PrivacySettingsSnapshot,
    ) -> impl Future<Output = Result<()>> + '_ {
        let work = async move { Result::Ok(()) };

        // On WASM, the work future is non-Send, because the HTTP request future contains a reference to a JS
        // value (which is fine, since our WASM executor is single-threaded). On all other platforms, we must
        // return a Send future in order to use the background executor.
        cfg_if::cfg_if! {
            if #[cfg(target_family = "wasm")] {
                work.boxed_local()
            } else {
                work.boxed()
            }
        }
    }
}
