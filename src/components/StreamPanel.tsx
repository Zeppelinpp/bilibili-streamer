import { useState, useEffect } from 'react';
import { useUser } from '@/context/AppContext';
import { useLive } from '@/context/AppContext';
import { useUI } from '@/context/AppContext';
import { startLive, stopLive, updateTitle, updateArea, getPartitions } from '@/hooks/useTauri';
import { QRCodeSVG } from 'qrcode.react';
import { X } from 'lucide-react';

export default function StreamPanel() {
  const { user } = useUser();
  const { isLive, setIsLive, streamCode, setStreamCode } = useLive();
  const { addLog } = useUI();
  const [title, setTitle] = useState(user?.last_title ?? '');
  const [partitions, setPartitions] = useState<Record<string, string[]>>({});
  const [parentArea, setParentArea] = useState('');
  const [subArea, setSubArea] = useState('');
  const [faceQrUrl, setFaceQrUrl] = useState<string | null>(null);

  useEffect(() => {
    if (user?.last_title) {
      setTitle(user.last_title);
    }
  }, [user?.last_title]);

  useEffect(() => {
    let cancelled = false;
    getPartitions()
      .then((data) => {
        if (cancelled) return;
        setPartitions(data);
        const parents = Object.keys(data);
        if (parents.length === 0) return;
        const firstParent = parents[0];
        setParentArea(firstParent);
        const subs = data[firstParent];
        if (subs && subs.length > 0) {
          setSubArea(subs[0]);
        }
      })
      .catch((e: any) => {
        if (cancelled) return;
        console.error('获取分区失败:', e);
      });
    return () => { cancelled = true; };
  }, []);

  useEffect(() => {
    if (Object.keys(partitions).length === 0) return;
    const saved = user?.last_area_name;
    if (saved && saved.length >= 2 && partitions[saved[0]]?.includes(saved[1])) {
      setParentArea(saved[0]);
      setSubArea(saved[1]);
    }
  }, [partitions, user?.last_area_name]);

  const handleParentChange = (p: string) => {
    setParentArea(p);
    const subs = partitions[p] || [];
    if (subs.length > 0) {
      setSubArea(subs[0]);
    } else {
      setSubArea('');
    }
  };

  const handleUpdateArea = async () => {
    if (!parentArea || !subArea) return;
    try {
      await updateArea(parentArea, subArea);
      addLog('分区已更新');
    } catch (e: any) {
      addLog(`更新分区失败: ${e}`);
    }
  };

  const handleStart = async () => {
    addLog('开始获取推流码...');
    try {
      const res = await startLive(parentArea || undefined, subArea || undefined);
      if (res.code === 60024 || res.code === 60043) {
        setFaceQrUrl(res.qr || null);
        addLog('需要人脸验证，请扫码完成认证');
        return;
      }
      if (res.code !== 0) {
        addLog(`开播失败: ${res.msg || '未知错误'}`);
        return;
      }
      if (res.data) {
        setStreamCode(res.data);
        setIsLive(true);
        addLog('开播成功！');
      }
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

          <div>
            <label className="block text-[13px] text-stone-600 dark:text-stone-400 mb-1.5">分区</label>
            <div className="flex gap-2">
              <select
                value={parentArea}
                onChange={(e) => handleParentChange(e.target.value)}
                className="flex-1 h-9 px-3 rounded-lg bg-stone-50 dark:bg-stone-900 border border-stone-200 dark:border-stone-800 text-[13px] focus:outline-none focus:ring-2 focus:ring-stone-400/30 transition appearance-none"
              >
                {Object.keys(partitions).map((p) => (
                  <option key={p} value={p}>{p}</option>
                ))}
              </select>
              <select
                value={subArea}
                onChange={(e) => setSubArea(e.target.value)}
                className="flex-1 h-9 px-3 rounded-lg bg-stone-50 dark:bg-stone-900 border border-stone-200 dark:border-stone-800 text-[13px] focus:outline-none focus:ring-2 focus:ring-stone-400/30 transition appearance-none"
              >
                {(partitions[parentArea] || []).map((s) => (
                  <option key={s} value={s}>{s}</option>
                ))}
              </select>
              <button
                onClick={handleUpdateArea}
                disabled={!parentArea || !subArea}
                className="h-9 px-4 rounded-lg bg-stone-800 dark:bg-stone-100 text-white dark:text-stone-900 text-[13px] font-medium hover:opacity-90 transition disabled:opacity-50"
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

      {/* Face Verification QR Modal */}
      {faceQrUrl && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 dark:bg-black/60 backdrop-blur-sm"
          onClick={() => setFaceQrUrl(null)}
        >
          <div
            className="relative w-72 p-6 rounded-xl bg-white dark:bg-stone-900 border border-stone-200 dark:border-stone-800 shadow-2xl"
            onClick={(e) => e.stopPropagation()}
          >
            <button
              onClick={() => setFaceQrUrl(null)}
              className="absolute top-3 right-3 w-6 h-6 flex items-center justify-center rounded-md text-stone-400 hover:text-stone-600 dark:hover:text-stone-300 hover:bg-stone-100 dark:hover:bg-stone-800 transition"
            >
              <X size={14} />
            </button>

            <div className="text-center mb-4">
              <h3 className="text-[15px] font-semibold text-stone-800 dark:text-stone-100">人脸认证</h3>
              <p className="text-[11px] text-stone-400 mt-1">请使用哔哩哔哩客户端扫码完成认证</p>
            </div>

            <div className="flex flex-col items-center gap-4">
              <div className="p-3 bg-white rounded-xl border border-stone-100 shadow-sm">
                <QRCodeSVG value={faceQrUrl} size={152} />
              </div>
              <div className="text-[11px] text-stone-400 text-center leading-relaxed">
                扫码完成后，请重新点击开始直播
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
