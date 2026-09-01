use crate::models::live::StartLiveResponse;
use crate::state::AppState;
use anyhow::Result;

/// Facade for orchestrated live-session operations.
///
/// Lock ordering rule (prevents deadlocks): config -> session -> live ->
/// danmaku. This facade owns every multi-lock live operation; each method
/// acquires the locks in this order and drops them promptly. `api` is not
/// part of the ordering: BiliApi guards its own mutable state internally
/// and is shared lock-free.
pub struct LiveSession<'a> {
    state: &'a AppState,
}

impl<'a> LiveSession<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }

    /// Start the stream, then connect the danmaku monitor on success.
    pub async fn start_live(
        &self,
        p_name: Option<String>,
        s_name: Option<String>,
    ) -> Result<StartLiveResponse> {
        let mut config = self.state.config.lock().await;
        let mut session = self.state.session.lock().await;
        let mut live = self.state.live.lock().await;
        let result = live
            .start_live(&self.state.api, &mut session, &mut config, p_name, s_name)
            .await?;

        if result.code == 0 {
            let room_id = session.room_id.clone();
            let uid = session.uid;
            drop(config);
            drop(session);
            drop(live);

            if let Some(room_id) = room_id {
                if let Ok(room_id_num) = room_id.parse::<u64>() {
                    let danmaku_opt = self.state.danmaku.lock().await;
                    if let Some(danmaku) = danmaku_opt.as_ref() {
                        if !danmaku.is_running().await {
                            danmaku.connect(room_id_num, uid).await;
                        }
                    }
                }
            }
        }

        Ok(result)
    }

    /// Stop the stream, then disconnect the danmaku monitor.
    pub async fn stop_live(&self) -> Result<()> {
        let mut session = self.state.session.lock().await;
        let mut live = self.state.live.lock().await;
        live.stop_live(&self.state.api, &mut session).await?;
        drop(session);
        drop(live);

        let danmaku_opt = self.state.danmaku.lock().await;
        if let Some(danmaku) = danmaku_opt.as_ref() {
            if danmaku.is_running().await {
                danmaku.disconnect().await;
            }
        }

        Ok(())
    }

    pub async fn update_title(&self, title: &str) -> Result<()> {
        let mut config = self.state.config.lock().await;
        let session = self.state.session.lock().await;
        crate::services::live_service::LiveService::update_title(
            &self.state.api,
            &session,
            &mut config,
            title,
        )
        .await
    }

    pub async fn update_area(&self, p_name: &str, s_name: &str) -> Result<()> {
        let mut config = self.state.config.lock().await;
        let mut session = self.state.session.lock().await;
        let mut live = self.state.live.lock().await;
        live.update_area(&self.state.api, &mut session, &mut config, p_name, s_name)
            .await
    }

    /// Stop the stream if still live; used by the exit path.
    pub async fn shutdown_cleanup(&self) {
        let mut session = self.state.session.lock().await;
        if session.is_live {
            let mut live = self.state.live.lock().await;
            if let Err(e) = live.stop_live(&self.state.api, &mut session).await {
                tracing::error!("Failed to stop live on exit: {}", e);
            }
        }
    }
}
