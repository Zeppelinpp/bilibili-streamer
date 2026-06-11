import { useUI } from '@/context/AppContext';
import { getVersion, setAppConfig, getAppConfig, checkUpdate, installUpdate } from '@/hooks/useTauri';
import { useState, useEffect } from 'react';

export default function SettingsPanel() {
  const { isDark, toggleDark } = useUI();
  const [minToTray, setMinToTray] = useState(true);
  const [version, setVersion] = useState('');
  const [checkingUpdate, setCheckingUpdate] = useState(false);
  const [updateStatus, setUpdateStatus] = useState('');

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

  const handleCheckUpdate = async () => {
    if (checkingUpdate) return;
    setCheckingUpdate(true);
    setUpdateStatus('正在检查更新...');
    try {
      const info = await checkUpdate();
      if (info.available && info.version) {
        const confirmed = window.confirm(
          `发现新版本 ${info.version}${info.body ? '\n\n' + info.body : ''}\n\n是否下载并重启应用？`
        );
        if (confirmed) {
          setUpdateStatus('正在下载更新...');
          await installUpdate((progress) => {
            setUpdateStatus(`正在下载更新... ${Math.round(progress * 100)}%`);
          });
        } else {
          setUpdateStatus('');
        }
      } else {
        setUpdateStatus('当前已是最新版本');
        setTimeout(() => setUpdateStatus(''), 2000);
      }
    } catch (e) {
      setUpdateStatus('检查更新失败');
      setTimeout(() => setUpdateStatus(''), 2000);
    } finally {
      setCheckingUpdate(false);
    }
  };

  return (
    <div className="flex-1 overflow-y-auto p-6">
      <div className="max-w-2xl space-y-8">
        <section>
          <h2 className="text-[11px] font-semibold uppercase tracking-wider text-stone-500 mb-5">偏好设置</h2>
          <div className="space-y-1">
            <div className="flex items-center justify-between py-3 border-b border-stone-200 dark:border-stone-800">
              <div>
                <div className="text-[13px] font-medium text-stone-800 dark:text-stone-200">关闭时最小化到托盘</div>
                <div className="text-[12px] text-stone-400 mt-0.5">点击关闭按钮将隐藏到系统托盘</div>
              </div>
              <button onClick={toggleMinToTray} className={`relative w-10 h-6 rounded-full transition ${minToTray ? 'bg-[#34C759]' : 'bg-stone-300 dark:bg-stone-600'}`}>
                <span className={`absolute top-1 w-4 h-4 rounded-full bg-white transition ${minToTray ? 'left-5' : 'left-1'}`} />
              </button>
            </div>
            <div className="flex items-center justify-between py-3 border-b border-stone-200 dark:border-stone-800">
              <div>
                <div className="text-[13px] font-medium text-stone-800 dark:text-stone-200">深色模式</div>
                <div className="text-[12px] text-stone-400 mt-0.5">切换应用主题</div>
              </div>
              <button onClick={toggleDark} className={`relative w-10 h-6 rounded-full transition ${isDark ? 'bg-[#34C759]' : 'bg-stone-300 dark:bg-stone-600'}`}>
                <span className={`absolute top-1 w-4 h-4 rounded-full bg-white transition ${isDark ? 'left-5' : 'left-1'}`} />
              </button>
            </div>
          </div>
        </section>
        <section>
          <h2 className="text-[11px] font-semibold uppercase tracking-wider text-stone-500 mb-5">关于</h2>
          <div className="flex items-center justify-between py-2">
            <span className="text-[12px] text-stone-500">版本</span>
            <span className="text-[12px] text-stone-400">{version}</span>
          </div>
          <div className="flex items-center justify-between py-2">
            <span className="text-[12px] text-stone-500">更新</span>
            <div className="flex items-center gap-2">
              {updateStatus && (
                <span className="text-[12px] text-stone-400">{updateStatus}</span>
              )}
              <button
                onClick={handleCheckUpdate}
                disabled={checkingUpdate}
                className="text-[12px] px-2.5 py-1 rounded-md bg-stone-100 dark:bg-stone-800 text-stone-600 dark:text-stone-300 hover:bg-stone-200 dark:hover:bg-stone-700 disabled:opacity-50 transition"
              >
                {checkingUpdate ? '检查中...' : '检查更新'}
              </button>
            </div>
          </div>
        </section>
      </div>
    </div>
  );
}
