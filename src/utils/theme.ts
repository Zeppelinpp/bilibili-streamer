import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

export type ThemeMode = "dark" | "light";

interface WindowThemeColor {
	r: number;
	g: number;
	b: number;
	a?: number;
}

/** Native window background colour per window label and mode. */
const WINDOW_THEME_COLORS: Record<string, Record<ThemeMode, WindowThemeColor>> =
	{
		main: {
			dark: { r: 45, g: 42, b: 46 },
			light: { r: 247, g: 245, b: 242 },
		},
		"danmaku-float": {
			dark: { r: 28, g: 26, b: 28, a: 204 },
			light: { r: 247, g: 245, b: 242, a: 204 },
		},
	};

/**
 * Apply a theme mode to the current window: toggle the `dark` class on the
 * document root and set the native window background colour from the table.
 */
export function applyTheme(dark: boolean) {
	document.documentElement.classList.toggle("dark", dark);
	const label = getCurrentWebviewWindow().label;
	const color = WINDOW_THEME_COLORS[label]?.[dark ? "dark" : "light"];
	if (!color) return;
	invoke("set_window_background", { ...color, dark }).catch(() => {});
}
