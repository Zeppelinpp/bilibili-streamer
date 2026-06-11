import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./styles/globals.css";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

if (getCurrentWebviewWindow().label === "danmaku-float") {
	document.body.classList.add("float-window");
}

const root = document.getElementById("root");
if (!root) throw new Error("Root element not found");
ReactDOM.createRoot(root).render(
	<React.StrictMode>
		<App />
	</React.StrictMode>,
);
