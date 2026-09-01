import { useCallback } from "react";
import DanmakuStreamView from "@/components/DanmakuStreamView";
import { useDanmaku, useUI, useUser } from "@/context/AppContext";

export default function DanmakuPanel() {
	const { danmakuList, clearDanmaku } = useDanmaku();
	const { addLog } = useUI();
	const { user } = useUser();

	const handleEmoteLoaded = useCallback(
		(map: Record<string, string>) => {
			if (Object.keys(map).length === 0) {
				addLog("[表情] 未获取到官方表情，将使用 unicode 兜底");
			}
		},
		[addLog],
	);

	return (
		<DanmakuStreamView
			className="flex-1 flex flex-col overflow-hidden"
			messages={danmakuList}
			user={user}
			onClear={clearDanmaku}
			onEmoteLoaded={handleEmoteLoaded}
			onError={addLog}
		/>
	);
}
