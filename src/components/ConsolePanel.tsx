import { useUI } from "@/context/AppContext";

interface ConsolePanelProps {
	open: boolean;
}

export default function ConsolePanel({ open }: ConsolePanelProps) {
	const { consoleLogs, clearLogs } = useUI();

	if (!open) return null;

	return (
		<div
			className="border-t border-stone-200 dark:border-stone-800 bg-stone-50 dark:bg-stone-950 flex flex-col shrink-0"
			style={{ height: 120 }}
		>
			<div className="flex items-center justify-between px-4 h-7 border-b border-stone-200 dark:border-stone-800">
				<span className="text-[11px] font-medium text-stone-500 uppercase tracking-wider">
					Console
				</span>
				<div className="flex gap-3">
					<button
						type="button"
						onClick={clearLogs}
						className="text-[11px] text-stone-500 hover:text-stone-700 dark:hover:text-stone-300 transition"
					>
						清空
					</button>
				</div>
			</div>
			<div className="flex-1 overflow-y-auto p-3 font-mono text-[11px] space-y-0.5 leading-relaxed">
				{consoleLogs.map((item) => (
					<div key={item.id} className="text-stone-500 dark:text-stone-400">
						{item.text}
					</div>
				))}
			</div>
		</div>
	);
}
