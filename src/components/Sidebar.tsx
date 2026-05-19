import { useState, useEffect, useRef } from 'react';
import { useUser } from '@/context/AppContext';
import { useLive } from '@/context/AppContext';
import { useUI } from '@/context/AppContext';
import { logout, clearSession } from '@/hooks/useTauri';
import { RadioTower, MessageSquare, User, Settings } from 'lucide-react';
import LoginPanel from './LoginPanel';

interface SidebarProps {
  activeTab: string;
  onTabChange: (tab: string) => void;
}

const navItems = [
  { id: 'stream', label: '推流设置', icon: RadioTower },
  { id: 'danmaku', label: '弹幕监控', icon: MessageSquare },
  { id: 'account', label: '账户管理', icon: User },
];

export default function Sidebar({ activeTab, onTabChange }: SidebarProps) {
  const { user, setUser } = useUser();
  const { isLive } = useLive();
  const { addLog } = useUI();
  const [showLogin, setShowLogin] = useState(false);
  const [showMenu, setShowMenu] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        setShowMenu(false);
      }
    };
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  const handleLogout = async () => {
    if (!user) return;
    try {
      await logout(user.uid);
      setUser(null);
      addLog('已退出登录');
    } catch (e: any) {
      addLog(`退出登录失败: ${e}`);
    }
    setShowMenu(false);
  };

  const handleSwitchUser = async () => {
    if (!user) return;
    try {
      await clearSession();
      setUser(null);
      addLog('已退出当前账户');
    } catch (e: any) {
      addLog(`切换用户失败: ${e}`);
    }
    setShowMenu(false);
    setShowLogin(true);
  };

  return (
    <div className="w-52 bg-stone-100 dark:bg-[#221f22] border-r border-stone-300 dark:border-[#4a454d] flex flex-col shrink-0">
      {/* User Card */}
      <div ref={menuRef} className="relative p-3 mb-2">
        <div
          onClick={() => {
            if (user) {
              setShowMenu((v) => !v);
            } else {
              setShowLogin((v) => !v);
            }
          }}
          className="flex items-center gap-2.5 p-2.5 rounded-lg transition cursor-pointer hover:bg-stone-200 dark:hover:bg-[#363236]"
        >
          <img
            src={user?.face || 'https://static.hdslb.com/images/member/noface.gif'}
            className="w-8 h-8 rounded-full object-cover"
            referrerPolicy="no-referrer"
            alt=""
          />
          <div className="flex-1 min-w-0">
            <div className="text-[14px] font-medium truncate text-stone-800 dark:text-stone-100">{user?.uname ?? '未登录'}</div>
            <div className="text-[11px] text-stone-400 truncate">
              {user ? `LV${user.level} · ${user.uid}` : '点击登录'}
            </div>
          </div>
        </div>

        {showMenu && user && (
          <div className="absolute left-3 right-3 top-full mt-1 py-1 rounded-lg bg-white dark:bg-stone-900 border border-stone-200 dark:border-stone-800 shadow-lg z-20">
            <button
              onClick={handleSwitchUser}
              className="w-full px-3 py-2 text-[12px] text-left text-stone-700 dark:text-stone-300 hover:bg-stone-100 dark:hover:bg-stone-800 transition"
            >
              切换用户
            </button>
            <button
              onClick={handleLogout}
              className="w-full px-3 py-2 text-[12px] text-left text-red-500 hover:bg-stone-100 dark:hover:bg-stone-800 transition"
            >
              退出登录
            </button>
          </div>
        )}

        {showLogin && !user && <LoginPanel onClose={() => setShowLogin(false)} />}
      </div>

      {/* Navigation */}
      <nav className="flex-1 px-2 space-y-0.5">
        {navItems.map((item) => {
          const isActive = activeTab === item.id;
          return (
            <button
              key={item.id}
              onClick={() => onTabChange(item.id)}
              className={`w-full flex items-center gap-3 px-3 py-2 rounded-lg text-[13px] font-medium transition ${
                isActive
                  ? 'bg-stone-200 dark:bg-[#363236] text-stone-900 dark:text-stone-100'
                  : 'text-stone-500 dark:text-stone-400 hover:bg-stone-200 dark:hover:bg-[#363236] hover:text-stone-900 dark:hover:text-stone-100'
              }`}
            >
              <item.icon size={16} />
              <span>{item.label}</span>
            </button>
          );
        })}

        <div className="pt-4 mt-4">
          <button
            onClick={() => onTabChange('settings')}
            className={`w-full flex items-center gap-3 px-3 py-2 rounded-lg text-[13px] font-medium transition ${
              activeTab === 'settings'
                ? 'bg-stone-200 dark:bg-[#363236] text-stone-900 dark:text-stone-100'
                : 'text-stone-500 dark:text-stone-400 hover:bg-stone-200 dark:hover:bg-[#363236] hover:text-stone-900 dark:hover:text-stone-100'
            }`}
          >
            <Settings size={16} />
            <span>设置</span>
          </button>
        </div>
      </nav>

      {/* Live Status */}
      <div className="px-3 pt-2">
        <div className="flex items-center gap-2 px-3 py-2">
          <span className={`w-1.5 h-1.5 rounded-full ${isLive ? 'bg-[#34C759] animate-pulse' : 'bg-stone-300 dark:bg-stone-600'}`} />
          <span className="text-[11px] text-stone-400">{isLive ? '直播中' : '未开播'}</span>
        </div>
      </div>
    </div>
  );
}
