import { type ReactNode, useState } from "react";

export const FALLBACK_EMOJI_MAP: Record<string, string> = {
	dog: "https://i0.hdslb.com/bfs/emote/3087d273a78ccaff4bb1e9972e2ba2a7583c9f11.png",
	doge: "https://i0.hdslb.com/bfs/emote/3087d273a78ccaff4bb1e9972e2ba2a7583c9f11.png",
	妙啊: "👍",
	吃瓜: "🍉",
	呲牙: "😁",
	打call: "📣",
	酸了: "🍋",
	大哭: "😭",
	喜极而泣: "😂",
	笑哭: "😂",
	偷笑: "🤭",
	爱心: "❤️",
	胜利: "✌️",
	保佑: "🙏",
	灵魂出窍: "😇",
	OK: "👌",
	点赞: "👍",
	捂脸: "🤦",
	尴尬: "😅",
	黑洞: "🕳️",
	跪了: "🧎",
	给心心: "🫶",
	惊讶: "😲",
	再见: "👋",
	惊喜: "🤩",
	鼓掌: "👏",
};

export function normalizeImageUrl(url?: string | null): string | null {
	if (!url) return null;
	if (url.startsWith("http://")) return `https://${url.slice(7)}`;
	return url.startsWith("https://") ? url : null;
}

function EmoteImage({ url, text }: { url: string; text: string }) {
	const [failed, setFailed] = useState(false);
	const normalizedUrl = normalizeImageUrl(url);

	if (failed || !normalizedUrl) return <span>{text}</span>;

	return (
		<img
			src={normalizedUrl}
			alt={text}
			className="inline-block w-5 h-5 object-contain align-text-bottom"
			loading="lazy"
			referrerPolicy="no-referrer"
			onError={() => setFailed(true)}
		/>
	);
}

export function parseMessage(
	msg: string,
	emoteMap: Record<string, string>,
	messageEmotes?: Record<string, string>,
): ReactNode[] {
	const localEmotes = messageEmotes ?? {};
	const standaloneUrl = localEmotes[msg];
	if (standaloneUrl) {
		return [
			<EmoteImage key="standalone-emote" url={standaloneUrl} text={msg} />,
		];
	}

	const segments: ReactNode[] = [];
	const regex = /\[([^\]]+)\]/g;
	let lastIndex = 0;
	let match: RegExpExecArray | null;
	let key = 0;

	while (true) {
		match = regex.exec(msg);
		if (match === null) break;
		const textBefore = msg.slice(lastIndex, match.index);
		if (textBefore) {
			segments.push(<span key={key++}>{textBefore}</span>);
		}

		const code = match[1];
		const fullCode = `[${code}]`;
		const url = localEmotes[fullCode] || emoteMap[fullCode];
		if (url) {
			segments.push(<EmoteImage key={key++} url={url} text={fullCode} />);
		} else if (FALLBACK_EMOJI_MAP[code]) {
			const fallback = FALLBACK_EMOJI_MAP[code];
			if (normalizeImageUrl(fallback)) {
				segments.push(
					<EmoteImage key={key++} url={fallback} text={fullCode} />,
				);
			} else {
				segments.push(<span key={key++}>{fallback}</span>);
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
