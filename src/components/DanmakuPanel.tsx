import { useEffect, useRef, useState, type ReactNode } from 'react';
import { useDanmaku } from '@/context/AppContext';
import { useUI } from '@/context/AppContext';
import { useUser } from '@/context/AppContext';
import { sendDanmaku, getEmoteList } from '@/hooks/useTauri';
import { Send, Trash2 } from 'lucide-react';

// B站直播颜文字（如 [dog]）不在 reply/dynamic 官方 API 中，是客户端硬编码的。
// 因此将已确认 URL 的颜文字手动映射，其余使用 unicode 兜底。
const FALLBACK_EMOJI_MAP: Record<string, string> = {
  dog: 'https://i0.hdslb.com/bfs/emote/3087d273a78ccaff4bb1e9972e2ba2a7583c9f11.png',
  妙啊: '👍',
  辣眼睛: '😵',
  吃瓜: '🍉',
  滑稽: '😏',
  呲牙: '😁',
  打call: '📣',
  歪嘴: '😏',
  酸了: '🍋',
  大哭: '😭',
  喜极而泣: '😂',
  笑哭: '😂',
  偷笑: '🤭',
  生气: '😠',
  无语: '😶',
  害羞: '😳',
  嫌弃: '😒',
  爱心: '❤️',
  胜利: '✌️',
  保佑: '🙏',
  灵魂出窍: '😇',
  OK: '👌',
  点赞: '👍',
  捂脸: '🤦',
  尴尬: '😅',
  黑洞: '🕳️',
  跪了: '🧎',
  给心心: '🫶',
  哦呼: '😮',
  嘟嘟: '😗',
  惊讶: '😲',
  再见: '👋',
  抠鼻: '🤧',
  惊喜: '🤩',
  鼓掌: '👏',
};

function parseMessage(msg: string, emoteMap: Record<string, string>): ReactNode[] {
  const segments: ReactNode[] = [];
  const regex = /\[([^\]]+)\]/g;
  let lastIndex = 0;
  let match: RegExpExecArray | null;
  let key = 0;

  while ((match = regex.exec(msg)) !== null) {
    const textBefore = msg.slice(lastIndex, match.index);
    if (textBefore) {
      segments.push(<span key={key++}>{textBefore}</span>);
    }

    const code = match[1];
    const fullCode = `[${code}]`;
    const url = emoteMap[fullCode];
    if (url && url.startsWith('http')) {
      segments.push(
        <img
          key={key++}
          src={url}
          alt={fullCode}
          className="inline-block w-5 h-5 align-text-bottom"
          loading="lazy"
        />
      );
    } else if (FALLBACK_EMOJI_MAP[code]) {
      const fb = FALLBACK_EMOJI_MAP[code];
      if (fb.startsWith('http')) {
        segments.push(
          <img
            key={key++}
            src={fb}
            alt={fullCode}
            className="inline-block w-5 h-5 align-text-bottom"
            loading="lazy"
          />
        );
      } else {
        segments.push(<span key={key++}>{fb}</span>);
      }
    } else {
      segments.push(<span key={key++}>{fullCode}</span>);
    }

    lastIndex = regex.lastIndex;
  }

  const textAfter = msg.slice(lastIndex);
  if (textAfter) {
    segments.push(<span key={key++}>{textAfter}</span>);
  }

  return segments;
}

export default function DanmakuPanel() {
  const { danmakuList, clearDanmaku } = useDanmaku();
  const { addLog } = useUI();
  const { user } = useUser();
  const [input, setInput] = useState('');
  const [emoteMap, setEmoteMap] = useState<Record<string, string>>({});
  const scrollRef = useRef<HTMLDivElement>(null);
  const isAtBottomRef = useRef(true);

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

  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      <div ref={scrollRef} onScroll={handleScroll} className="flex-1 overflow-y-auto px-6 py-3 space-y-1">
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
            msgClass = 'bg-stone-700 text-white dark:bg-stone-200 dark:text-stone-900';
          } else if (item.data.type === 'gift') {
            msgClass = 'bg-amber-50 text-amber-700 dark:bg-amber-900/40 dark:text-amber-400';
          } else {
            msgClass = 'bg-stone-100 text-stone-800 dark:bg-stone-800 dark:text-stone-200';
          }
          return (
            <div key={item.id} className={`flex py-1.5 px-3 rounded-lg transition ${isSelf ? 'justify-end' : 'justify-start'}`}>
              <div className={`flex items-start gap-2 max-w-[85%] ${isSelf ? 'flex-row-reverse' : 'flex-row'}`}>
                {item.data.uname && (
                  <span className="text-[12px] font-medium text-stone-500 mt-1 shrink-0">
                    {item.data.uname}
                  </span>
                )}
                <span className={`text-[13px] px-3 py-1.5 rounded-lg ${msgClass}`}>
                  {parseMessage(item.data.msg || '', emoteMap)}
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
