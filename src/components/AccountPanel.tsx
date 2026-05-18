import { useUser } from '@/context/AppContext';
import { useUI } from '@/context/AppContext';
import { getAccountList, switchAccount, logout } from '@/hooks/useTauri';
import { useState, useEffect } from 'react';
import type { UserConfig } from '@/types/api';

export default function AccountPanel() {
  const { user, setUser } = useUser();
  const { addLog } = useUI();
  const [accounts, setAccounts] = useState<UserConfig[]>([]);

  useEffect(() => {
    getAccountList()
      .then(setAccounts)
      .catch((e: any) => addLog(`获取账户列表失败: ${e}`));
  }, [addLog]);

  const handleSwitch = async (uid: number) => {
    try {
      const u = await switchAccount(uid);
      setUser(u);
      addLog(`已切换到账户: ${u.uname}`);
    } catch (e: any) {
      addLog(`切换账户失败: ${e}`);
    }
  };

  const handleLogout = async (uid: number) => {
    try {
      await logout(uid);
      setAccounts((prev) => prev.filter((a) => a.uid !== uid));
      if (user?.uid === uid) setUser(null);
      addLog('账户已删除');
    } catch (e: any) {
      addLog(`删除账户失败: ${e}`);
    }
  };

  return (
    <div className="flex-1 overflow-y-auto p-6 space-y-6">
      <section>
        <h2 className="text-[11px] font-semibold uppercase tracking-wider text-stone-400 mb-3">当前账户</h2>
        {user && (
          <div className="p-4 rounded-xl bg-stone-50 dark:bg-stone-900 border border-stone-200 dark:border-stone-800 flex items-center gap-3">
            <div className="w-10 h-10 rounded-full bg-stone-400 flex items-center justify-center text-white font-semibold">{user.uname[0]}</div>
            <div className="flex-1">
              <div className="text-[13px] font-medium">{user.uname}</div>
              <div className="text-[12px] text-stone-400">UID: {user.uid}</div>
            </div>
            <span className="text-[11px] px-2 py-0.5 rounded-full bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-400 border border-green-200 dark:border-green-800">已登录</span>
          </div>
        )}
      </section>

      <section>
        <h2 className="text-[11px] font-semibold uppercase tracking-wider text-stone-400 mb-3">已保存的账户</h2>
        <div className="space-y-2">
          {accounts.map((acc) => (
            <div key={acc.uid} className="flex items-center gap-3 p-3 rounded-xl bg-stone-50 dark:bg-stone-900 border border-stone-200 dark:border-stone-800">
              <div className="w-8 h-8 rounded-full bg-stone-300 dark:bg-stone-700 flex items-center justify-center text-white text-xs font-semibold">{acc.uname[0]}</div>
              <div className="flex-1 min-w-0">
                <div className="text-[13px]">{acc.uname}</div>
                <div className="text-[12px] text-stone-400">UID: {acc.uid}</div>
              </div>
              <button onClick={() => handleSwitch(acc.uid)} className="px-3 h-7 rounded-md text-[12px] bg-stone-800 dark:bg-stone-100 text-white dark:text-stone-900 hover:opacity-90 transition">切换</button>
              <button onClick={() => handleLogout(acc.uid)} className="px-3 h-7 rounded-md text-[12px] text-stone-400 hover:text-red-500 transition">删除</button>
            </div>
          ))}
        </div>
      </section>
    </div>
  );
}
