import { Gift } from "lucide-react";
import { useState } from "react";
import type { DanmakuMessage } from "@/types/api";
import { normalizeImageUrl } from "@/utils/danmaku";

interface GiftNoticeProps {
	data: DanmakuMessage;
	compact?: boolean;
}

export default function GiftNotice({ data, compact = false }: GiftNoticeProps) {
	const [imageFailed, setImageFailed] = useState(false);
	const iconUrl = normalizeImageUrl(data.gift_icon);
	const iconSize = compact ? 16 : 18;

	return (
		<div
			className={`flex justify-center ${compact ? "py-1 px-2" : "py-1.5 px-3"}`}
		>
			<span
				className={`inline-flex items-center gap-1 text-amber-600 dark:text-amber-400 ${compact ? "text-[12px]" : "text-[13px]"}`}
			>
				{data.uname && (
					<span className="font-medium text-stone-900 dark:text-stone-100">
						{data.uname}
					</span>
				)}
				<span>赠送了</span>
				{iconUrl && !imageFailed ? (
					<img
						src={iconUrl}
						alt={data.gift_name || "礼物"}
						className="inline-block object-contain"
						style={{ width: iconSize, height: iconSize }}
						loading="lazy"
						referrerPolicy="no-referrer"
						onError={() => setImageFailed(true)}
					/>
				) : (
					<Gift size={iconSize} aria-hidden="true" />
				)}
				<span className="font-medium">{data.gift_name || "礼物"}</span>
				<span>×{data.num ?? 0}</span>
			</span>
		</div>
	);
}
