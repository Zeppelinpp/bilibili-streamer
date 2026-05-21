import { useEffect, useRef, useState } from 'react';
import { useDanmaku, useUI, useUser } from '@/context/AppContext';
import { sendDanmaku, closeDanmakuFloat, getEmoteList } from '@/hooks/useTauri';
import { Send, Trash2, X } from 'lucide-react';
import { parseMessage } from '@/utils/danmaku';
import { invoke } from '@tauri-apps/api/core';

export default function DanmakuFloat() {
  const { danmakuList, clearDanmaku } = useDanmaku();
  const { addLog } = useUI();
  const { user } = useUser();
  const [input, setInput] = useState('');
  const [emoteMap, setEmoteMap] = useState<Record<string, string>>({});
  const scrollRef = useRef<HTMLDivElement>(null);
  const isAtBottomRef = useRef(true);

  useEffect(() => {
    document.documentElement.style.background = 'transparent';
    document.body.classList.add('float-window');

    const mq = window.matchMedia('(prefers-color-scheme: dark)');
    const applyTheme = (e: MediaQueryList | MediaQueryListEvent) => {
      if (e.matches) {
        document.documentElement.classList.add('dark');
      } else {
        document.documentElement.classList.remove('dark');
      }
    };
    applyTheme(mq);
    mq.addEventListener('change', applyTheme);
    return () => {
      mq.removeEventListener('change', applyTheme);
      document.body.classList.remove('float-window');
    };
  }, []);

  useEffect(() => {
    if (!user) return;
    getEmoteList()
      .then((map) => {
        setEmoteMap(map);
        if (Object.keys(map).length === 0) {
          addLog('[表情] 未获取到官方表情，将使用 unicode 兜底');
        }
      })
      .catch((e) => {
        addLog(`[表情] 获取官方表情失败: ${e}`);
      });
  }, [user, addLog]);

  useEffect(() => {
    if (scrollRef.current && isAtBottomRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [danmakuList]);

  const handleScroll = () => {
    const el = scrollRef.current;
    if (!el) return;
    isAtBottomRef.current = el.scrollHeight - el.scrollTop - el.clientHeight < 30;
  };

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

  const handleClose = async () => {
    try {
      await closeDanmakuFloat();
    } catch (e: any) {
      addLog(`[浮窗] 关闭失败: ${e}`);
    }
  };

  return (
    <div className="flex flex-col h-screen bg-stone-950/70 text-stone-200 overflow-hidden select-none rounded-xl">
      {/* Drag handle / title bar */}
      <div
        className="flex items-center justify-between px-3 h-7 shrink-0 bg-stone-900/60 border-b border-stone-800/50 cursor-grab active:cursor-grabbing"
        onMouseDown={() => {
          invoke('window_drag', { x: 0, y: 0 }).catch(() => {});
        }}
      >
        <span className="text-[11px] font-medium text-stone-400">Monitor</span>
        <div className="flex items-center gap-1">
          <button
            onClick={handleClose}
            className="w-5 h-5 rounded flex items-center justify-center text-stone-500 hover:text-stone-200 hover:bg-stone-800 transition"
            title="关闭"
          >
            <X size={11} />
          </button>
        </div>
      </div>

      {/* Danmaku list */}
      <div
        ref={scrollRef}
        onScroll={handleScroll}
        className="flex-1 overflow-y-auto px-3 py-2 space-y-1"
      >
        {danmakuList.map((item) => {
          const isSelf = item.data.is_self;
          if (item.data.type === 'interact') {
            const uname = item.data.uname || '';
            const rest = (item.data.msg || '').replace(uname, '').trimStart();
            return (
              <div key={item.id} className="flex justify-center py-1 px-2">
                <span className="text-[11px] text-stone-500">
                  {uname && (
                    <span className="font-medium text-stone-300">{uname}</span>
                  )}
                  {uname && ' '}
                  {rest}
                </span>
              </div>
            );
          }
          let msgClass: string;
          if (isSelf) {
            msgClass = 'bg-stone-600/90 text-white';
          } else if (item.data.type === 'gift') {
            msgClass = 'bg-amber-900/40 text-amber-400';
          } else {
            msgClass = 'bg-stone-800/80 text-stone-200';
          }
          return (
            <div
              key={item.id}
              className={`flex py-1 px-2 rounded-md transition ${isSelf ? 'justify-end' : 'justify-start'}`}
            >
              <div className={`flex items-start gap-1.5 max-w-[90%] ${isSelf ? 'flex-row-reverse' : 'flex-row'}`}>
                {item.data.uname && (
                  <span className="text-[11px] font-medium text-stone-500 mt-0.5 shrink-0">
                    {item.data.uname}
                  </span>
                )}
                <span className={`text-[12px] px-2 py-1 rounded-md ${msgClass}`}>
                  {parseMessage(item.data.msg || '', emoteMap)}
                </span>
              </div>
            </div>
          );
        })}
      </div>

      {/* Input bar */}
      <div className="px-3 py-2 shrink-0 border-t border-stone-800 bg-stone-950/80">
        <div className="flex gap-1.5">
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
            className="flex-1 h-7 px-2 rounded-md bg-stone-900 border border-stone-800 text-[12px] text-stone-200 placeholder:text-stone-600 focus:outline-none focus:ring-1 focus:ring-stone-600 transition"
          />
          <button
            onClick={clearDanmaku}
            className="w-7 h-7 rounded-md flex items-center justify-center text-stone-500 hover:text-stone-300 hover:bg-stone-800 transition"
            title="清空"
          >
            <Trash2 size={12} />
          </button>
          <button
            onClick={handleSend}
            className="w-7 h-7 rounded-md flex items-center justify-center bg-[#D4652A] text-white hover:opacity-90 transition"
            title="发送"
          >
            <Send size={12} />
          </button>
        </div>
      </div>
    </div>
  );
}
