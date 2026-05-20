import { createContext, useCallback, useContext, useEffect, useRef, useState, type ReactNode } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import type { DanmakuMessage, StreamCodeData, StreamProtocolType, UserConfig } from '@/types/api';

// ---------- UserContext ----------
interface UserState {
  user: UserConfig | null;
  setUser: (user: UserConfig | null) => void;
}

const UserContext = createContext<UserState | null>(null);

function UserProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<UserConfig | null>(null);
  return (
    <UserContext.Provider value={{ user, setUser }}>
      {children}
    </UserContext.Provider>
  );
}

export function useUser() {
  const ctx = useContext(UserContext);
  if (!ctx) throw new Error('useUser must be used within AppProvider');
  return ctx;
}

// ---------- LiveContext ----------
interface LiveState {
  isLive: boolean;
  setIsLive: (v: boolean) => void;
  streamCode: StreamCodeData | null;
  setStreamCode: (v: StreamCodeData | null) => void;
  selectedProtocol: StreamProtocolType;
  setSelectedProtocol: (v: StreamProtocolType) => void;
}

const LiveContext = createContext<LiveState | null>(null);

function LiveProvider({ children }: { children: ReactNode }) {
  const [isLive, setIsLive] = useState(false);
  const [streamCode, setStreamCode] = useState<StreamCodeData | null>(null);
  const [selectedProtocol, setSelectedProtocol] = useState<StreamProtocolType>('rtmp1');
  return (
    <LiveContext.Provider value={{ isLive, setIsLive, streamCode, setStreamCode, selectedProtocol, setSelectedProtocol }}>
      {children}
    </LiveContext.Provider>
  );
}

export function useLive() {
  const ctx = useContext(LiveContext);
  if (!ctx) throw new Error('useLive must be used within AppProvider');
  return ctx;
}

// ---------- DanmakuContext ----------
export interface DanmakuItem {
  id: number;
  data: DanmakuMessage;
}

interface DanmakuState {
  danmakuList: DanmakuItem[];
  addDanmaku: (msg: DanmakuMessage) => void;
  clearDanmaku: () => void;
}

const DanmakuContext = createContext<DanmakuState | null>(null);

function DanmakuProvider({ children }: { children: ReactNode }) {
  const [danmakuList, setDanmakuList] = useState<DanmakuItem[]>([]);
  const nextId = useRef(0);

  const addDanmaku = useCallback((msg: DanmakuMessage) => {
    setDanmakuList((prev) => [...prev, { id: nextId.current++, data: msg }].slice(-500));
  }, []);

  const clearDanmaku = useCallback(() => setDanmakuList([]), []);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let cancelled = false;
    listen('danmu-message', (event) => {
      const msg = event.payload as DanmakuMessage;
      setDanmakuList((prev) => [...prev, { id: nextId.current++, data: msg }].slice(-500));
    })
      .then((fn) => {
        if (cancelled) {
          fn();
        } else {
          unlisten = fn;
        }
      })
      .catch((err) => {
        console.error('Failed to listen for danmu-message:', err);
      });
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  return (
    <DanmakuContext.Provider value={{ danmakuList, addDanmaku, clearDanmaku }}>
      {children}
    </DanmakuContext.Provider>
  );
}

export function useDanmaku() {
  const ctx = useContext(DanmakuContext);
  if (!ctx) throw new Error('useDanmaku must be used within AppProvider');
  return ctx;
}

// ---------- UIContext ----------
export interface LogItem {
  id: number;
  text: string;
}

interface UIState {
  consoleLogs: LogItem[];
  addLog: (log: string) => void;
  clearLogs: () => void;
  isDark: boolean;
  setIsDark: (v: boolean) => void;
  consoleOpen: boolean;
  setConsoleOpen: (v: boolean) => void;
}

const UIContext = createContext<UIState | null>(null);

function UIProvider({ children }: { children: ReactNode }) {
  const [consoleLogs, setConsoleLogs] = useState<LogItem[]>([]);
  const [isDark, setIsDark] = useState(true);
  const [consoleOpen, setConsoleOpen] = useState(false);
  const nextId = useRef(0);

  useEffect(() => {
    const mq = window.matchMedia('(prefers-color-scheme: dark)');
    const initialDark = mq.matches;
    setIsDark(initialDark);
    if (initialDark) {
      document.documentElement.classList.add('dark');
      invoke('set_window_background', { r: 45, g: 42, b: 46 }).catch(() => {});
    } else {
      document.documentElement.classList.remove('dark');
      invoke('set_window_background', { r: 247, g: 245, b: 242 }).catch(() => {});
    }

    const handler = (e: MediaQueryList | MediaQueryListEvent) => {
      setIsDark(e.matches);
      if (e.matches) {
        document.documentElement.classList.add('dark');
        invoke('set_window_background', { r: 45, g: 42, b: 46 }).catch(() => {});
      } else {
        document.documentElement.classList.remove('dark');
        invoke('set_window_background', { r: 247, g: 245, b: 242 }).catch(() => {});
      }
    };
    mq.addListener(handler);
    return () => mq.removeListener(handler);
  }, []);

  const addLog = useCallback((log: string) => {
    setConsoleLogs((prev) => [...prev, { id: nextId.current++, text: log }].slice(-200));
  }, []);

  const clearLogs = useCallback(() => setConsoleLogs([]), []);

  return (
    <UIContext.Provider value={{ consoleLogs, addLog, clearLogs, isDark, setIsDark, consoleOpen, setConsoleOpen }}>
      {children}
    </UIContext.Provider>
  );
}

export function useUI() {
  const ctx = useContext(UIContext);
  if (!ctx) throw new Error('useUI must be used within AppProvider');
  return ctx;
}

// ---------- Combined AppProvider ----------
export function AppProvider({ children }: { children: ReactNode }) {
  return (
    <UserProvider>
      <LiveProvider>
        <DanmakuProvider>
          <UIProvider>{children}</UIProvider>
        </DanmakuProvider>
      </LiveProvider>
    </UserProvider>
  );
}
