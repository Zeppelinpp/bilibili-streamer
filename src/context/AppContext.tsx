import { createContext, useContext, useState, type ReactNode } from 'react';
import type { DanmakuMessage, StreamCodeData, UserConfig } from '@/types/api';

interface AppState {
  user: UserConfig | null;
  setUser: (user: UserConfig | null) => void;
  isLive: boolean;
  setIsLive: (v: boolean) => void;
  streamCode: StreamCodeData | null;
  setStreamCode: (v: StreamCodeData | null) => void;
  danmakuList: DanmakuMessage[];
  addDanmaku: (msg: DanmakuMessage) => void;
  clearDanmaku: () => void;
  consoleLogs: string[];
  addLog: (log: string) => void;
  clearLogs: () => void;
  isDark: boolean;
  setIsDark: (v: boolean) => void;
  consoleOpen: boolean;
  setConsoleOpen: (v: boolean) => void;
}

const AppContext = createContext<AppState | null>(null);

export function AppProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<UserConfig | null>(null);
  const [isLive, setIsLive] = useState(false);
  const [streamCode, setStreamCode] = useState<StreamCodeData | null>(null);
  const [danmakuList, setDanmakuList] = useState<DanmakuMessage[]>([]);
  const [consoleLogs, setConsoleLogs] = useState<string[]>([]);
  const [isDark, setIsDark] = useState(true);
  const [consoleOpen, setConsoleOpen] = useState(true);

  const addDanmaku = (msg: DanmakuMessage) => {
    setDanmakuList((prev) => [...prev, msg].slice(-500));
  };

  const clearDanmaku = () => setDanmakuList([]);

  const addLog = (log: string) => {
    setConsoleLogs((prev) => [...prev, log].slice(-200));
  };

  const clearLogs = () => setConsoleLogs([]);

  return (
    <AppContext.Provider
      value={{
        user, setUser,
        isLive, setIsLive,
        streamCode, setStreamCode,
        danmakuList, addDanmaku, clearDanmaku,
        consoleLogs, addLog, clearLogs,
        isDark, setIsDark,
        consoleOpen, setConsoleOpen,
      }}
    >
      {children}
    </AppContext.Provider>
  );
}

export function useApp() {
  const ctx = useContext(AppContext);
  if (!ctx) throw new Error('useApp must be used within AppProvider');
  return ctx;
}
