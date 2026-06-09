import { useState, useEffect } from 'react';
import { AppProvider } from '@/context/AppContext';
import { useUI } from '@/context/AppContext';
import { useUser } from '@/context/AppContext';
import { loadSavedConfig } from '@/hooks/useTauri';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import Sidebar from '@/components/Sidebar';
import StreamPanel from '@/components/StreamPanel';
import DanmakuPanel from '@/components/DanmakuPanel';
import DanmakuFloat from '@/components/DanmakuFloat';
import AccountPanel from '@/components/AccountPanel';
import SettingsPanel from '@/components/SettingsPanel';
import ConsolePanel from '@/components/ConsolePanel';
import { Sun, Moon, Terminal } from 'lucide-react';

function AppContent() {
  const [activeTab, setActiveTab] = useState('stream');
  const { isDark, setIsDark, consoleOpen, setConsoleOpen } = useUI();
  const { setUser } = useUser();

  useEffect(() => {
    loadSavedConfig()
      .then((u) => {
        if (u) setUser(u);
      })
      .catch(() => {});
  }, [setUser]);

  const renderPanel = () => {
    switch (activeTab) {
      case 'stream': return <StreamPanel />;
      case 'danmaku': return <DanmakuPanel />;
      case 'account': return <AccountPanel />;
      case 'settings': return <SettingsPanel />;
      default: return <StreamPanel />;
    }
  };

  return (
    <div className="flex h-screen bg-[#f7f5f2] text-stone-800 dark:bg-stone-950 dark:text-stone-200 overflow-hidden">
      <Sidebar activeTab={activeTab} onTabChange={setActiveTab} />
      <div className="flex-1 flex flex-col min-w-0">
        <div className="flex items-center justify-end px-4 h-10 gap-2">
          <button
            onClick={() => {
              const nextDark = !isDark;
              setIsDark(nextDark);
              if (nextDark) {
                document.documentElement.classList.add('dark');
                invoke('set_window_background', { r: 45, g: 42, b: 46, dark: true }).catch(() => {});
              } else {
                document.documentElement.classList.remove('dark');
                invoke('set_window_background', { r: 247, g: 245, b: 242, dark: false }).catch(() => {});
              }
            }}
            className="w-8 h-8 rounded-md flex items-center justify-center text-stone-500 dark:text-stone-300 hover:text-stone-700 dark:hover:text-stone-200 hover:bg-stone-200 dark:hover:bg-[#363236] transition"
          >
            {isDark ? <Sun size={14} /> : <Moon size={14} />}
          </button>
          <button
            onClick={() => setConsoleOpen(!consoleOpen)}
            className={`w-8 h-8 rounded-md flex items-center justify-center transition ${consoleOpen ? 'text-stone-800 dark:text-stone-200 bg-stone-200 dark:bg-[#363236]' : 'text-stone-500 dark:text-stone-300 hover:text-stone-700 dark:hover:text-stone-200 hover:bg-stone-200 dark:hover:bg-[#363236]'}`}
          >
            <Terminal size={14} />
          </button>
        </div>
        {renderPanel()}
        <ConsolePanel open={consoleOpen} />
      </div>
    </div>
  );
}

function AppInner() {
  const label = getCurrentWebviewWindow().label;
  if (label === 'danmaku-float') {
    return <DanmakuFloat />;
  }
  return <AppContent />;
}

function App() {
  return (
    <AppProvider>
      <AppInner />
    </AppProvider>
  );
}

export default App;
