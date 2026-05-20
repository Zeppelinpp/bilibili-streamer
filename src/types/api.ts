export interface UserConfig {
  uid: number;
  uname: string;
  face: string;
  room_id: string;
  csrf: string;
  last_title: string;
  last_area_id: number;
  last_area_name: string[];
  level: number;
  follower: number;
  following: number;
  dynamic_count: number;
}

export interface StreamProtocol {
  addr: string;
  code: string;
}

export type StreamProtocolType = 'rtmp1' | 'rtmp2' | 'srt';

export interface StreamCodeData {
  rtmp1: StreamProtocol;
  rtmp2: StreamProtocol;
  srt: StreamProtocol;
}

export interface QrCodeData {
  url: string;
  qrcode_key: string;
}

export interface LoginResult {
  code: number;
  uid?: number;
  user?: UserConfig;
}

export interface DanmakuMessage {
  type: 'danmaku' | 'interact' | 'gift' | 'system';
  uid?: number;
  uname?: string;
  face?: string;
  msg?: string;
  gift_name?: string;
  num?: number;
  action?: string;
  is_self?: boolean;
}

export interface AppConfig {
  min_to_tray: boolean;
}

export type PartitionMap = Record<string, string[]>;

export interface StartLiveResponse {
  code: number;
  data?: StreamCodeData;
  qr?: string;
  msg?: string;
}
