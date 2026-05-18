import { createContext, useContext, useRef, useState, type ReactNode } from 'react';
import type { DanmakuMessage, StreamCodeData, UserConfig } from '@/types/api';

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
}

const LiveContext = createContext<LiveState | null>(null);

function LiveProvider({ children }: { children: ReactNode }) {
  const [isLive, setIsLive] = useState(false);
  const [streamCode, setStreamCode] = useState<StreamCodeData | null>(null);
  return (
    <LiveContext.Provider value={{ isLive, setIsLive, streamCode, setStreamCode }}>
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

  const addDanmaku = (msg: DanmakuMessage) => {
    setDanmakuList((prev) => [...prev, { id: nextId.current++, data: msg }].slice(-500));
  };

  const clearDanmaku = () => setDanmakuList([]);

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
  const [consoleOpen, setConsoleOpen] = useState(true);
  const nextId = useRef(0);

  const addLog = (log: string) => {
    setConsoleLogs((prev) => [...prev, { id: nextId.current++, text: log }].slice(-200));
  };

  const clearLogs = () => setConsoleLogs([]);

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
