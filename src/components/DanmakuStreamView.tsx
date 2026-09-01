import { Send, Trash2 } from "lucide-react";
import { memo, useEffect, useRef, useState } from "react";
import type { DanmakuItem } from "@/context/AppContext";
import { getEmoteList, sendDanmaku } from "@/hooks/useTauri";
import type { UserConfig } from "@/types/api";
import { parseMessage } from "@/utils/danmaku";
import GiftNotice from "./GiftNotice";

interface DanmakuStreamItemProps {
	item: DanmakuItem;
	emoteMap: Record<string, string>;
	compact: boolean;
}

const DanmakuStreamItem = memo(function DanmakuStreamItem({
	item,
	emoteMap,
	compact,
}: DanmakuStreamItemProps) {
	const isSelf = item.data.is_self;
	if (item.data.type === "gift") {
		return <GiftNotice data={item.data} compact={compact} />;
	}
	if (item.data.type === "interact") {
		const uname = item.data.uname || "";
		const rest = (item.data.msg || "").replace(uname, "").trimStart();
		return (
			<div
				className={`flex justify-center ${compact ? "py-1 px-2" : "py-1.5 px-3"}`}
			>
				<span
					className={
						compact
							? "text-[12px] text-stone-500 dark:text-stone-400"
							: "text-[13px] text-stone-400"
					}
				>
					{uname && (
						<span
							className={
								compact
									? "font-medium text-stone-800 dark:text-stone-200"
									: "font-medium text-stone-900 dark:text-stone-100"
							}
						>
							{uname}
						</span>
					)}
					{uname && " "}
					{rest}
				</span>
			</div>
		);
	}
	const msgClass = isSelf
		? "bg-stone-700 text-white dark:bg-stone-200 dark:text-stone-900"
		: compact
			? "bg-white/90 text-stone-800 shadow-sm dark:bg-[#646064]/90 dark:text-stone-200 dark:shadow-none"
			: "bg-white text-stone-800 shadow dark:bg-[#646064] dark:text-stone-200 dark:shadow-none";
	return (
		<div
			className={`flex rounded-lg transition ${compact ? "py-1 px-2" : "py-1.5 px-3"} ${isSelf ? "justify-end" : "justify-start"}`}
		>
			<div
				className={`flex items-start ${compact ? "gap-1.5 max-w-[90%]" : "gap-2 max-w-[85%]"} ${isSelf ? "flex-row-reverse" : "flex-row"}`}
			>
				{item.data.uname && (
					<span
						className={`font-medium shrink-0 ${
							compact
								? "text-[11px] text-stone-500 dark:text-stone-400 mt-0.5"
								: "text-[12px] text-stone-600 dark:text-stone-400 mt-1"
						}`}
					>
						{item.data.uname}
					</span>
				)}
				<span
					className={`rounded-lg ${compact ? "text-[12px] px-2.5 py-1" : "text-[13px] px-3 py-1.5"} ${msgClass}`}
				>
					{parseMessage(item.data.msg || "", emoteMap, item.data.emotes)}
				</span>
			</div>
		</div>
	);
});

interface DanmakuStreamViewProps {
	/** Layout classes for the outer container, supplied by the shell. */
	className: string;
	messages: DanmakuItem[];
	user: UserConfig | null;
	onClear: () => void;
	/** Compact density for the float Monitor window. */
	compact?: boolean;
	/** Called after the official emote map loads (used by the main panel to log an empty map). */
	onEmoteLoaded?: (map: Record<string, string>) => void;
	/** Called on emote-load and send failures; omit for silent failures (float window). */
	onError?: (message: string) => void;
}

export default function DanmakuStreamView({
	className,
	messages,
	user,
	onClear,
	compact = false,
	onEmoteLoaded,
	onError,
}: DanmakuStreamViewProps) {
	const [input, setInput] = useState("");
	const [emoteMap, setEmoteMap] = useState<Record<string, string>>({});
	const scrollRef = useRef<HTMLDivElement>(null);
	const isAtBottomRef = useRef(true);
	// Callbacks are read through refs so the emote-loading effect and send
	// handler never depend on the shells' callback identity.
	const onEmoteLoadedRef = useRef(onEmoteLoaded);
	onEmoteLoadedRef.current = onEmoteLoaded;
	const onErrorRef = useRef(onError);
	onErrorRef.current = onError;

	useEffect(() => {
		if (!user) return;
		getEmoteList()
			.then((map) => {
				setEmoteMap(map);
				onEmoteLoadedRef.current?.(map);
			})
			.catch((e) => {
				onErrorRef.current?.(`[表情] 获取官方表情失败: ${e}`);
			});
	}, [user]);

	useEffect(() => {
		if (messages.length === 0) return;
		if (scrollRef.current && isAtBottomRef.current) {
			scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
		}
	}, [messages]);

	const handleScroll = () => {
		const el = scrollRef.current;
		if (!el) return;
		isAtBottomRef.current = el.scrollHeight - el.scrollTop - el.clientHeight < 30;
	};

	const handleSend = async () => {
		const text = input.trim();
		if (!text) return;
		try {
			const res = await sendDanmaku(text);
			if (res.code !== 0) {
				onErrorRef.current?.(`[弹幕] 发送失败: ${res.msg}`);
			}
			if (res.code === 0) setInput("");
		} catch (e) {
			onErrorRef.current?.(`[弹幕] 发送失败: ${e}`);
		}
	};

	const controlSize = compact ? "w-8 h-8" : "w-9 h-9";
	const iconSize = compact ? 14 : 15;

	return (
		<div className={className}>
			<div
				ref={scrollRef}
				onScroll={handleScroll}
				className={`flex-1 overflow-y-auto space-y-1 ${compact ? "px-4 pt-8 pb-2" : "px-6 py-3"}`}
			>
				{messages.map((item) => (
					<DanmakuStreamItem
						key={item.id}
						item={item}
						emoteMap={emoteMap}
						compact={compact}
					/>
				))}
			</div>
			<div
				className={`shrink-0 ${
					compact
						? "px-4 py-3 border-t border-stone-200/40 dark:border-stone-700/40"
						: "px-6 py-4"
				}`}
			>
				<div className="flex gap-2">
					<input
						type="text"
						value={input}
						onChange={(e) => setInput(e.target.value)}
						onKeyDown={(e) => {
							if (e.key === "Enter") {
								e.preventDefault();
								handleSend();
							}
						}}
						placeholder="发送弹幕..."
						className={`flex-1 rounded-lg focus:outline-none focus:ring-2 focus:ring-stone-400/30 transition ${
							compact
								? "h-8 px-2.5 bg-white/70 dark:bg-stone-900/70 border border-stone-200/60 dark:border-stone-800/60 text-[12px]"
								: "h-9 px-3 bg-stone-50 dark:bg-stone-900 border border-stone-200 dark:border-stone-800 text-[13px]"
						}`}
					/>
					<button
						type="button"
						onClick={onClear}
						className={`${controlSize} rounded-lg flex items-center justify-center transition ${
							compact
								? "text-stone-500 dark:text-stone-400 hover:text-stone-700 dark:hover:text-stone-200 hover:bg-stone-200/70 dark:hover:bg-[#363236]/70"
								: "text-stone-500 dark:text-stone-300 hover:text-stone-700 dark:hover:text-stone-200 hover:bg-stone-200 dark:hover:bg-[#363236]"
						}`}
						title="清空"
					>
						<Trash2 size={iconSize} />
					</button>
					<button
						type="button"
						onClick={handleSend}
						className={`${controlSize} rounded-lg flex items-center justify-center bg-[#D4652A] text-white hover:opacity-90 transition`}
						title="发送"
					>
						<Send size={iconSize} />
					</button>
				</div>
			</div>
		</div>
	);
}
