import { useEffect } from "react";
import DanmakuStreamView from "@/components/DanmakuStreamView";
import { useDanmaku, useUser } from "@/context/AppContext";

export default function DanmakuFloat() {
	const { danmakuList, clearDanmaku } = useDanmaku();
	const { user } = useUser();

	useEffect(() => {
		document.body.classList.add("float-window");
		return () => {
			document.body.classList.remove("float-window");
		};
	}, []);

	return (
		<DanmakuStreamView
			className="h-screen flex flex-col overflow-hidden"
			messages={danmakuList}
			user={user}
			onClear={clearDanmaku}
			compact
		/>
	);
}
