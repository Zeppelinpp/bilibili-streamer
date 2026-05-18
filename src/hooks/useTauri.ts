import { invoke } from '@tauri-apps/api/core';
import type { AppConfig, LoginResult, PartitionMap, QrCodeData, StreamCodeData, UserConfig } from '@/types/api';

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

export async function getPartitions(): Promise<PartitionMap> {
  return await invoke('get_partitions');
}

export async function updateTitle(title: string): Promise<void> {
  return await invoke('update_title', { title });
}

export async function updateArea(pName: string, sName: string): Promise<void> {
  return await invoke('update_area', { pName, sName });
}

export async function startLive(pName?: string, sName?: string): Promise<StreamCodeData> {
  return await invoke('start_live', { pName, sName });
}

export async function stopLive(): Promise<void> {
  return await invoke('stop_live');
}

export async function sendDanmaku(msg: string): Promise<{ code: number; msg: string }> {
  return await invoke('send_danmaku', { msg });
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

export async function windowMin(): Promise<void> {
  return await invoke('window_min');
}

export async function windowMax(): Promise<boolean> {
  return await invoke('window_max');
}

export async function windowClose(): Promise<void> {
  return await invoke('window_close');
}
