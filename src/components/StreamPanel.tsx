import { useState, useEffect } from 'react';
import { useUser } from '@/context/AppContext';
import { useLive } from '@/context/AppContext';
import { useUI } from '@/context/AppContext';
import { startLive, stopLive, updateTitle } from '@/hooks/useTauri';

export default function StreamPanel() {
  const { user } = useUser();
  const { isLive, setIsLive, streamCode, setStreamCode } = useLive();
  const { addLog } = useUI();
  const [title, setTitle] = useState(user?.last_title ?? '');

  useEffect(() => {
    if (user?.last_title) {
      setTitle(user.last_title);
    }
  }, [user?.last_title]);

  const handleStart = async () => {
    addLog('开始获取推流码...');
    try {
      const data = await startLive();
      setStreamCode(data);
      setIsLive(true);
      addLog('开播成功！');
    } catch (e: any) {
      addLog(`开播失败: ${e}`);
    }
  };

  const handleStop = async () => {
    addLog('正在停止直播...');
    try {
      await stopLive();
      setIsLive(false);
      setStreamCode(null);
      addLog('直播已停止');
    } catch (e: any) {
      addLog(`停止失败: ${e}`);
    }
  };

  return (
    <div className="flex-1 overflow-y-auto p-6 space-y-6">
      {/* Title & Area */}
      <section>
        <h2 className="text-[11px] font-semibold uppercase tracking-wider text-stone-400 mb-3">直播信息</h2>
        <div className="space-y-4">
          <div>
            <label className="block text-[13px] text-stone-600 dark:text-stone-400 mb-1.5">标题</label>
            <div className="flex gap-2">
              <input
                type="text"
                value={title}
                onChange={(e) => setTitle(e.target.value)}
                className="flex-1 h-9 px-3 rounded-lg bg-stone-50 dark:bg-stone-900 border border-stone-200 dark:border-stone-800 text-[13px] focus:outline-none focus:ring-2 focus:ring-stone-400/30 transition"
              />
              <button
                onClick={async () => {
                  try {
                    await updateTitle(title);
                    addLog('标题已更新');
                  } catch (e: any) {
                    addLog(`更新标题失败: ${e}`);
                  }
                }}
                className="h-9 px-4 rounded-lg bg-stone-800 dark:bg-stone-100 text-white dark:text-stone-900 text-[13px] font-medium hover:opacity-90 transition"
              >
                更新
              </button>
            </div>
          </div>
        </div>
      </section>

      {/* Stream Codes */}
      {streamCode && (
        <section>
          <h2 className="text-[11px] font-semibold uppercase tracking-wider text-stone-400 mb-3">推流码</h2>
          <div className="space-y-3">
            {(['rtmp1', 'rtmp2', 'srt'] as const).map((key) => (
              <div key={key} className="group p-4 rounded-xl bg-stone-50 dark:bg-stone-900 border border-stone-200 dark:border-stone-800 hover:border-stone-300 dark:hover:border-stone-700 transition">
                <div className="flex items-center justify-between mb-2">
                  <span className="text-[12px] font-medium text-stone-500 uppercase">{key}</span>
                  <button
                    onClick={() => navigator.clipboard.writeText(`${streamCode[key].addr}${streamCode[key].code}`)}
                    className="text-[12px] text-stone-400 hover:text-stone-700 dark:hover:text-stone-300 transition opacity-0 group-hover:opacity-100"
                  >
                    复制
                  </button>
                </div>
                <code className="block text-[12px] text-stone-600 dark:text-stone-400 font-mono break-all leading-relaxed">
                  {streamCode[key].addr}{streamCode[key].code}
                </code>
              </div>
            ))}
          </div>
        </section>
      )}

      {/* Actions */}
      <div className="flex gap-3 pt-2">
        <button
          onClick={handleStart}
          disabled={isLive}
          className="flex-1 h-10 rounded-lg bg-stone-800 dark:bg-stone-100 text-white dark:text-stone-900 text-[13px] font-medium hover:opacity-90 transition disabled:opacity-50"
        >
          开始直播
        </button>
        <button
          onClick={handleStop}
          disabled={!isLive}
          className="flex-1 h-10 rounded-lg bg-stone-100 dark:bg-stone-900 border border-stone-200 dark:border-stone-800 text-[13px] font-medium hover:bg-stone-200 dark:hover:bg-stone-800 transition disabled:opacity-50"
        >
          停止直播
        </button>
      </div>
    </div>
  );
}
