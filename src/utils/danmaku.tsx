import type { ReactNode } from "react";

export const FALLBACK_EMOJI_MAP: Record<string, string> = {
	dog: "https://i0.hdslb.com/bfs/emote/3087d273a78ccaff4bb1e9972e2ba2a7583c9f11.png",
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

export function parseMessage(
	msg: string,
	emoteMap: Record<string, string>,
): ReactNode[] {
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
		const url = emoteMap[fullCode];
		if (url?.startsWith("http")) {
			segments.push(
				<img
					key={key++}
					src={url}
					alt={fullCode}
					className="inline-block w-5 h-5 align-text-bottom"
					loading="lazy"
				/>,
			);
		} else if (FALLBACK_EMOJI_MAP[code]) {
			const fb = FALLBACK_EMOJI_MAP[code];
			if (fb.startsWith("http")) {
				segments.push(
					<img
						key={key++}
						src={fb}
						alt={fullCode}
						className="inline-block w-5 h-5 align-text-bottom"
						loading="lazy"
					/>,
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
