import { useState, useEffect } from 'react';
import { useDanmaku } from '@/context/AppContext';
import { useUI } from '@/context/AppContext';
import { sendDanmaku, startDanmakuMonitor, stopDanmakuMonitor } from '@/hooks/useTauri';
import { listen } from '@tauri-apps/api/event';
import type { DanmakuMessage } from '@/types/api';

export default function DanmakuPanel() {
  const { danmakuList, addDanmaku, clearDanmaku } = useDanmaku();
  const { addLog } = useUI();
  const [input, setInput] = useState('');

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    const setup = async () => {
      try {
        await startDanmakuMonitor();
        unlisten = await listen('danmu-message', (event) => {
          addDanmaku(event.payload as DanmakuMessage);
        });
      } catch (e: any) {
        addLog(`[弹幕] 启动监听失败: ${e}`);
      }
    };
    setup();

    return () => {
      unlisten?.();
      stopDanmakuMonitor().catch(() => {});
    };
  }, [addDanmaku, addLog]);

  const handleSend = async () => {
    if (!input.trim()) return;
    try {
      const res = await sendDanmaku(input);
      addLog(`[弹幕] ${res.msg}`);
      if (res.code === 0) setInput('');
    } catch (e: any) {
      addLog(`[弹幕] 发送失败: ${e}`);
    }
  };

  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      <div className="flex items-center justify-between px-6 py-4 border-b border-stone-200 dark:border-stone-800 shrink-0">
        <h2 className="text-[13px] font-medium">弹幕消息</h2>
        <button onClick={clearDanmaku} className="text-[11px] text-stone-400 hover:text-stone-600 dark:hover:text-stone-300 transition">清空</button>
      </div>
      <div className="flex-1 overflow-y-auto px-6 py-3 space-y-1">
        {danmakuList.map((item) => (
          <div key={item.id} className="flex items-start gap-3 py-2 px-3 rounded-lg hover:bg-stone-50 dark:hover:bg-stone-900 transition">
            {item.data.uname && <span className="text-[12px] font-medium text-stone-500 mt-0.5 shrink-0">{item.data.uname}</span>}
            <span className={`text-[13px] ${item.data.type === 'gift' ? 'text-amber-600 dark:text-amber-500' : item.data.type === 'interact' ? 'text-stone-400' : 'text-stone-800 dark:text-stone-200'}`}>
              {item.data.msg}
            </span>
          </div>
        ))}
      </div>
      <div className="px-6 py-4 border-t border-stone-200 dark:border-stone-800 shrink-0">
        <div className="flex gap-2">
          <input
            type="text"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') {
                e.preventDefault();
                handleSend();
              }
            }}
            placeholder="发送弹幕..."
            className="flex-1 h-9 px-3 rounded-lg bg-stone-50 dark:bg-stone-900 border border-stone-200 dark:border-stone-800 text-[13px] focus:outline-none focus:ring-2 focus:ring-stone-400/30 transition"
          />
          <button onClick={handleSend} className="h-9 px-5 rounded-lg bg-stone-800 dark:bg-stone-100 text-white dark:text-stone-900 text-[13px] font-medium hover:opacity-90 transition">发送</button>
        </div>
      </div>
    </div>
  );
}
