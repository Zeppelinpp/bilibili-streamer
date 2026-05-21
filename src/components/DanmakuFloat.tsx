import { useEffect, useRef, useState } from 'react';
import { useDanmaku } from '@/context/AppContext';
import { useUser } from '@/context/AppContext';
import { sendDanmaku, getEmoteList } from '@/hooks/useTauri';
import { Send, Trash2 } from 'lucide-react';
import { parseMessage } from '@/utils/danmaku';

export default function DanmakuFloat() {
  const { danmakuList, clearDanmaku } = useDanmaku();
  const { user } = useUser();
  const [input, setInput] = useState('');
  const [emoteMap, setEmoteMap] = useState<Record<string, string>>({});
  const scrollRef = useRef<HTMLDivElement>(null);
  const isAtBottomRef = useRef(true);

  useEffect(() => {
    document.body.classList.add('float-window');
    return () => {
      document.body.classList.remove('float-window');
    };
  }, []);

  useEffect(() => {
    if (!user) return;
    getEmoteList()
      .then((map) => setEmoteMap(map))
      .catch(() => {});
  }, [user]);

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
      if (res.code === 0) setInput('');
    } catch {
      // silently fail in float window
    }
  };

  return (
    <div className="h-screen flex flex-col overflow-hidden">
      <div
        ref={scrollRef}
        onScroll={handleScroll}
        className="flex-1 overflow-y-auto px-4 pt-8 pb-2 space-y-1"
      >
        {danmakuList.map((item) => {
          const isSelf = item.data.is_self;
          if (item.data.type === 'interact') {
            const uname = item.data.uname || '';
            const rest = (item.data.msg || '').replace(uname, '').trimStart();
            return (
              <div key={item.id} className="flex justify-center py-1 px-2">
                <span className="text-[12px] text-stone-500 dark:text-stone-400">
                  {uname && (
                    <span className="font-medium text-stone-800 dark:text-stone-200">
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
            msgClass = 'bg-stone-700 text-white dark:bg-stone-200 dark:text-stone-900';
          } else if (item.data.type === 'gift') {
            msgClass = 'bg-amber-50 text-amber-700 dark:bg-amber-900/40 dark:text-amber-400';
          } else {
            msgClass =
              'bg-white/90 text-stone-800 shadow-sm dark:bg-[#646064]/90 dark:text-stone-200 dark:shadow-none';
          }
          return (
            <div
              key={item.id}
              className={`flex py-1 px-2 rounded-lg transition ${isSelf ? 'justify-end' : 'justify-start'}`}
            >
              <div
                className={`flex items-start gap-1.5 max-w-[90%] ${isSelf ? 'flex-row-reverse' : 'flex-row'}`}
              >
                {item.data.uname && (
                  <span className="text-[11px] font-medium text-stone-500 dark:text-stone-400 mt-0.5 shrink-0">
                    {item.data.uname}
                  </span>
                )}
                <span className={`text-[12px] px-2.5 py-1 rounded-lg ${msgClass}`}>
                  {parseMessage(item.data.msg || '', emoteMap)}
                </span>
              </div>
            </div>
          );
        })}
      </div>
      <div className="px-4 py-3 shrink-0 border-t border-stone-200/40 dark:border-stone-700/40">
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
            className="flex-1 h-8 px-2.5 rounded-lg bg-white/70 dark:bg-stone-900/70 border border-stone-200/60 dark:border-stone-800/60 text-[12px] focus:outline-none focus:ring-2 focus:ring-stone-400/30 transition"
          />
          <button
            onClick={clearDanmaku}
            className="w-8 h-8 rounded-lg flex items-center justify-center text-stone-500 dark:text-stone-400 hover:text-stone-700 dark:hover:text-stone-200 hover:bg-stone-200/70 dark:hover:bg-[#363236]/70 transition"
            title="清空"
          >
            <Trash2 size={14} />
          </button>
          <button
            onClick={handleSend}
            className="w-8 h-8 rounded-lg flex items-center justify-center bg-[#D4652A] text-white hover:opacity-90 transition"
            title="发送"
          >
            <Send size={14} />
          </button>
        </div>
      </div>
    </div>
  );
}
