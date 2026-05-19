import { useState, useEffect, useRef } from 'react';
import { QRCodeSVG } from 'qrcode.react';
import { getLoginQrcode, pollLoginStatus, refreshCurrentUser } from '@/hooks/useTauri';
import { useUser } from '@/context/AppContext';
import { useUI } from '@/context/AppContext';
import { X, RotateCcw } from 'lucide-react';

interface LoginPanelProps {
  onClose: () => void;
}

export default function LoginPanel({ onClose }: LoginPanelProps) {
  const { setUser } = useUser();
  const { addLog } = useUI();
  const [qrUrl, setQrUrl] = useState('');
  const [qrKey, setQrKey] = useState('');
  const [status, setStatus] = useState<'loading' | 'waiting' | 'scanned' | 'expired' | 'error'>('loading');
  const [errorMsg, setErrorMsg] = useState('');
  const mountedRef = useRef(true);

  useEffect(() => {
    mountedRef.current = true;
    setStatus('loading');
    setErrorMsg('');
    getLoginQrcode()
      .then((data) => {
        if (!mountedRef.current) return;
        setQrUrl(data.url);
        setQrKey(data.qrcode_key);
        setStatus('waiting');
      })
      .catch((e: any) => {
        if (!mountedRef.current) return;
        setStatus('error');
        setErrorMsg(e.toString());
      });
    return () => {
      mountedRef.current = false;
    };
  }, []);

  useEffect(() => {
    if (!qrKey) return;
    mountedRef.current = true;
    let interval: ReturnType<typeof setInterval>;

    const poll = async () => {
      if (!mountedRef.current) return;
      try {
        const res = await pollLoginStatus(qrKey);
        if (!mountedRef.current) return;
        if (res.code === 0) {
          clearInterval(interval);
          setStatus('scanned');
          addLog('扫码成功，正在获取用户信息...');
          try {
            const user = await refreshCurrentUser();
            if (!mountedRef.current) return;
            setUser(user);
            addLog(`登录成功: ${user.uname}`);
            onClose();
          } catch (e: any) {
            if (!mountedRef.current) return;
            setStatus('error');
            setErrorMsg(`获取用户信息失败: ${e}`);
          }
        } else if (res.code === 86038) {
          clearInterval(interval);
          setStatus('expired');
        } else if (res.code === 86090) {
          setStatus('scanned');
        } else {
          setStatus('waiting');
        }
      } catch (e: any) {
        if (!mountedRef.current) return;
        clearInterval(interval);
        setStatus('error');
        setErrorMsg(e.toString());
      }
    };

    poll();
    interval = setInterval(poll, 3000);
    return () => {
      mountedRef.current = false;
      clearInterval(interval);
    };
  }, [qrKey, addLog, setUser, onClose]);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [onClose]);

  const handleRefresh = () => {
    mountedRef.current = true;
    setStatus('loading');
    setErrorMsg('');
    getLoginQrcode()
      .then((data) => {
        if (!mountedRef.current) return;
        setQrUrl(data.url);
        setQrKey(data.qrcode_key);
        setStatus('waiting');
      })
      .catch((e: any) => {
        if (!mountedRef.current) return;
        setStatus('error');
        setErrorMsg(e.toString());
      });
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 dark:bg-black/60 backdrop-blur-sm"
      onClick={onClose}
    >
      <div
        className="relative w-72 p-6 rounded-xl bg-white dark:bg-stone-900 border border-stone-200 dark:border-stone-800 shadow-2xl"
        onClick={(e) => e.stopPropagation()}
      >
        <button
          onClick={onClose}
          className="absolute top-3 right-3 w-6 h-6 flex items-center justify-center rounded-md text-stone-400 hover:text-stone-600 dark:hover:text-stone-300 hover:bg-stone-100 dark:hover:bg-stone-800 transition"
        >
          <X size={14} />
        </button>

        <div className="text-center mb-4">
          <h3 className="text-[15px] font-semibold text-stone-800 dark:text-stone-100">扫码登录</h3>
          <p className="text-[11px] text-stone-400 mt-1">
            {status === 'waiting' && '请使用哔哩哔哩客户端扫码登录'}
            {status === 'scanned' && '已扫码，请在手机上确认登录'}
            {status === 'expired' && '二维码已过期'}
            {status === 'error' && '登录出错'}
            {status === 'loading' && '正在获取二维码...'}
          </p>
        </div>

        <div className="flex flex-col items-center gap-4">
          {status === 'loading' && (
            <div className="w-40 h-40 flex items-center justify-center">
              <div className="w-5 h-5 border-2 border-stone-300 dark:border-stone-600 border-t-stone-800 dark:border-t-stone-200 rounded-full animate-spin" />
            </div>
          )}

          {(status === 'waiting' || status === 'scanned') && qrUrl && (
            <div className={`p-3 bg-white rounded-xl border border-stone-100 shadow-sm transition-opacity ${status === 'scanned' ? 'opacity-60' : ''}`}>
              <QRCodeSVG value={qrUrl} size={152} />
            </div>
          )}

          {status === 'expired' && (
            <div className="w-40 h-40 flex flex-col items-center justify-center gap-2">
              <span className="text-[12px] text-stone-400">二维码已过期</span>
              <button
                onClick={handleRefresh}
                className="flex items-center gap-1.5 px-3 h-8 rounded-md text-[12px] bg-[#D4652A] text-white hover:opacity-90 transition"
              >
                <RotateCcw size={12} />
                刷新二维码
              </button>
            </div>
          )}

          {status === 'error' && (
            <div className="w-40 h-40 flex items-center justify-center px-2">
              <span className="text-[12px] text-red-500 text-center leading-relaxed">{errorMsg}</span>
            </div>
          )}

          <div className="text-[11px] text-stone-400 text-center leading-relaxed min-h-[1rem]">
            {status === 'waiting' && '打开哔哩哔哩 App → 我的 → 扫一扫'}
            {status === 'scanned' && '请在手机上点击确认登录'}
          </div>
        </div>
      </div>
    </div>
  );
}
