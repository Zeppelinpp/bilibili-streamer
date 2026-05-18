import { useUI } from '@/context/AppContext';
import { getVersion, setAppConfig, getAppConfig } from '@/hooks/useTauri';
import { useState, useEffect } from 'react';

export default function SettingsPanel() {
  const { isDark, setIsDark } = useUI();
  const [minToTray, setMinToTray] = useState(true);
  const [version, setVersion] = useState('');

  useEffect(() => {
    getVersion().then(setVersion).catch(() => {});
    getAppConfig()
      .then((cfg) => setMinToTray(cfg.min_to_tray))
      .catch((e: any) => console.error('获取配置失败:', e));
  }, []);

  const toggleMinToTray = async () => {
    const next = !minToTray;
    setMinToTray(next);
    await setAppConfig('min_to_tray', next);
  };

  return (
    <div className="flex-1 overflow-y-auto p-6">
      <div className="max-w-md space-y-6">
        <section>
          <h2 className="text-[11px] font-semibold uppercase tracking-wider text-stone-400 mb-4">偏好设置</h2>
          <div className="space-y-1">
            <div className="flex items-center justify-between py-3 border-b border-stone-100 dark:border-stone-900">
              <div>
                <div className="text-[13px]">关闭时最小化到托盘</div>
                <div className="text-[12px] text-stone-400 mt-0.5">点击关闭按钮将隐藏到系统托盘</div>
              </div>
              <button onClick={toggleMinToTray} className={`relative w-10 h-6 rounded-full transition ${minToTray ? 'bg-stone-800 dark:bg-stone-200' : 'bg-stone-200 dark:bg-stone-700'}`}>
                <span className={`absolute top-1 w-4 h-4 rounded-full bg-white transition ${minToTray ? 'left-5' : 'left-1'}`} />
              </button>
            </div>
            <div className="flex items-center justify-between py-3 border-b border-stone-100 dark:border-stone-900">
              <div>
                <div className="text-[13px]">深色模式</div>
                <div className="text-[12px] text-stone-400 mt-0.5">切换应用主题</div>
              </div>
              <button onClick={() => { setIsDark(!isDark); document.documentElement.classList.toggle('dark'); }} className={`relative w-10 h-6 rounded-full transition ${isDark ? 'bg-stone-800 dark:bg-stone-200' : 'bg-stone-200 dark:bg-stone-700'}`}>
                <span className={`absolute top-1 w-4 h-4 rounded-full bg-white transition ${isDark ? 'left-5' : 'left-1'}`} />
              </button>
            </div>
          </div>
        </section>
        <section>
          <h2 className="text-[11px] font-semibold uppercase tracking-wider text-stone-400 mb-4">关于</h2>
          <div className="flex items-center justify-between py-2">
            <span className="text-[13px] text-stone-500">版本</span>
            <span className="text-[13px] text-stone-400">{version}</span>
          </div>
        </section>
      </div>
    </div>
  );
}
