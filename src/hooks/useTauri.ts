import { invoke } from '@tauri-apps/api/core';
import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import type { AppConfig, LoginResult, PartitionMap, QrCodeData, StartLiveResponse, UserConfig } from '@/types/api';

export interface UpdateInfo {
  available: boolean;
  version?: string;
  body?: string;
  date?: string;
}

export async function checkUpdate(): Promise<UpdateInfo> {
  const update = await check();
  if (update) {
    return {
      available: true,
      version: update.version,
      body: update.body,
      date: update.date,
    };
  }
  return { available: false };
}

export async function installUpdate(onProgress?: (progress: number) => void): Promise<void> {
  const update = await check();
  if (!update) return;

  let total = 0;
  let downloaded = 0;

  await update.downloadAndInstall((event) => {
    switch (event.event) {
      case 'Started':
        total = event.data.contentLength ?? 0;
        break;
      case 'Progress':
        downloaded += event.data.chunkLength;
        if (total > 0 && onProgress) {
          onProgress(downloaded / total);
        }
        break;
      case 'Finished':
        if (onProgress) onProgress(1);
        break;
    }
  });
  await relaunch();
}

export async function getLoginQrcode(): Promise<QrCodeData> {
  return await invoke('get_login_qrcode');
}

export async function pollLoginStatus(key: string): Promise<LoginResult> {
  return await invoke('poll_login_status', { key });
}

export async function loadSavedConfig(): Promise<UserConfig | null> {
  return await invoke('load_saved_config');
}

export async function refreshCurrentUser(): Promise<UserConfig> {
  return await invoke('refresh_current_user');
}

export async function getAccountList(): Promise<UserConfig[]> {
  return await invoke('get_account_list');
}

export async function switchAccount(uid: number): Promise<UserConfig> {
  return await invoke('switch_account', { uid });
}

export async function logout(uid: number): Promise<void> {
  return await invoke('logout', { uid });
}

export async function clearSession(): Promise<void> {
  return await invoke('clear_session');
}

export async function getPartitions(): Promise<PartitionMap> {
  return await invoke('get_partitions');
}

export async function updateTitle(title: string): Promise<void> {
  return await invoke('update_title', { title });
}

export async function updateArea(pName: string, sName: string): Promise<void> {
  return await invoke('update_area', { pName, sName });
}

export async function startLive(pName?: string, sName?: string): Promise<StartLiveResponse> {
  return await invoke('start_live', { pName, sName });
}

export async function stopLive(): Promise<void> {
  return await invoke('stop_live');
}

export async function sendDanmaku(msg: string): Promise<{ code: number; msg: string }> {
  return await invoke('send_danmaku', { msg });
}

export async function getEmoteList(): Promise<Record<string, string>> {
  return await invoke('get_emote_list');
}

export async function getAppConfig(): Promise<AppConfig> {
  return await invoke('get_app_config');
}

export async function setAppConfig(key: string, value: boolean): Promise<void> {
  return await invoke('set_app_config', { key, value });
}

export async function getVersion(): Promise<string> {
  return await invoke('get_version');
}

export async function startDanmakuMonitor(): Promise<void> {
  return await invoke('start_danmaku_monitor');
}

export async function stopDanmakuMonitor(): Promise<void> {
  return await invoke('stop_danmaku_monitor');
}

export async function windowMin(): Promise<void> {
  return await invoke('window_min');
}

export async function windowMax(): Promise<boolean> {
  return await invoke('window_max');
}

export async function windowClose(): Promise<void> {
  return await invoke('window_close');
}

export async function windowDrag(x: number, y: number): Promise<void> {
  return await invoke('window_drag', { x, y });
}

export async function openDanmakuFloat(): Promise<void> {
  return await invoke('open_danmaku_float');
}

export async function closeDanmakuFloat(): Promise<void> {
  return await invoke('close_danmaku_float');
}
