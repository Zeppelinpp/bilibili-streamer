import { useState } from 'react';
import { useDanmaku } from '@/context/AppContext';
import { useUI } from '@/context/AppContext';
import { sendDanmaku } from '@/hooks/useTauri';
import { Send, Trash2 } from 'lucide-react';

export default function DanmakuPanel() {
  const { danmakuList, clearDanmaku } = useDanmaku();
  const { addLog } = useUI();
  const [input, setInput] = useState('');

  const handleSend = async () => {
    if (!input.trim()) return;
    try {
      const res = await sendDanmaku(input.trim());
      if (res.code !== 0) {
        addLog(`[弹幕] 发送失败: ${res.msg}`);
      }
      if (res.code === 0) setInput('');
    } catch (e: any) {
      addLog(`[弹幕] 发送失败: ${e}`);
    }
  };

  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      <div className="flex-1 overflow-y-auto px-6 py-3 space-y-1">
        {danmakuList.map((item) => {
          const isSelf = item.data.is_self;
          if (item.data.type === 'interact') {
            const uname = item.data.uname || '';
            const rest = (item.data.msg || '').replace(uname, '').trimStart();
            return (
              <div key={item.id} className="flex justify-center py-1.5 px-3">
                <span className="text-[13px] text-stone-400">
                  {uname && (
                    <span className="font-medium text-stone-900 dark:text-stone-100">
                      {uname}
                    </span>
                  )}
                  {uname && ' '}
                  {rest}
                </span>
              </div>
            );
          }
          let msgClass: string;
          if (isSelf) {
            msgClass = 'bg-stone-800 text-white dark:bg-stone-100 dark:text-stone-900';
          } else if (item.data.type === 'gift') {
            msgClass = 'text-amber-600 dark:text-amber-500';
          } else {
            msgClass = 'text-stone-800 dark:text-stone-200';
          }
          return (
            <div key={item.id} className={`flex py-2 px-3 rounded-lg hover:bg-stone-100 dark:hover:bg-stone-900 transition ${isSelf ? 'justify-end' : 'justify-start'}`}>
              <div className={`flex items-start gap-3 max-w-[85%] ${isSelf ? 'flex-row-reverse' : 'flex-row'}`}>
                {item.data.uname && (
                  <span className="text-[12px] font-medium text-stone-500 mt-0.5 shrink-0">
                    {item.data.uname}
                  </span>
                )}
                <span className={`text-[13px] px-3 py-1.5 rounded-lg ${msgClass}`}>
                  {item.data.msg}
                </span>
              </div>
            </div>
          );
        })}
      </div>
      <div className="px-6 py-4 shrink-0">
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
          <button
            onClick={clearDanmaku}
            className="w-9 h-9 rounded-lg flex items-center justify-center text-stone-400 hover:text-stone-600 dark:hover:text-stone-300 hover:bg-stone-100 dark:hover:bg-stone-900 transition"
            title="清空"
          >
            <Trash2 size={15} />
          </button>
          <button
            onClick={handleSend}
            className="w-9 h-9 rounded-lg flex items-center justify-center bg-stone-800 dark:bg-stone-100 text-white dark:text-stone-900 hover:opacity-90 transition"
            title="发送"
          >
            <Send size={15} />
          </button>
        </div>
      </div>
    </div>
  );
}
