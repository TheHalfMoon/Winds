import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import { markT143NativeReadyWindow } from "./leftDock/bridge";
import "./styles.css";
import "./fixture-modes.css";

const root = document.getElementById("root");

if (!root) {
  throw new Error("Winds desktop root element is missing");
}

const params = new URLSearchParams(window.location.search);
const documentRoot = document.documentElement;
const leftTool = params.get("tool");
if (leftTool === "chat" || leftTool === "projects") {
  documentRoot.dataset.leftTool = leftTool;
} else {
  delete documentRoot.dataset.leftTool;
}

const theme = params.get("theme");

if (theme === "dark" || theme === "light" || theme === "contrast") {
  documentRoot.dataset.theme = theme;
} else {
  delete documentRoot.dataset.theme;
}

if (params.get("fixture") === "keyboard-focus") {
  documentRoot.dataset.fixture = "keyboard-focus";
}

const scale = params.get("scale");
if (scale === "125" || scale === "200") {
  documentRoot.dataset.scale = scale;
}

if (params.get("motion") === "reduced") {
  documentRoot.dataset.motion = "reduced";
}

if (params.get("density") === "compact") {
  documentRoot.dataset.density = "compact";
}

if (params.get("layout") === "narrow") {
  documentRoot.dataset.layout = "narrow";
}

createRoot(root).render(
  <StrictMode>
    <App />
  </StrictMode>,
);


if (import.meta.env.VITE_WINDS_T143_NATIVE_READY === "1") {
  let reported = false;
  const markReady = () => {
    const status = document.querySelector<HTMLElement>(".chat-tool-status");
    const sessionOptions = document.querySelectorAll("#chat-session-target option");
    const visibleSlots = document.querySelectorAll(".session-slot");
    if (
      !document.querySelector(".winds-app")
      || !status
      || /Loading|Binding/.test(status.textContent ?? "")
      || sessionOptions.length < 3
      || visibleSlots.length < 2
    ) return false;
    window.requestAnimationFrame(() => {
      window.requestAnimationFrame(() => {
        if (reported) return;
        reported = true;
        void markT143NativeReadyWindow().catch((error: unknown) => {
          console.error("T143 native ready title failed", error);
        });
      });
    });
    return true;
  };
  const observer = new MutationObserver(() => {
    if (markReady()) observer.disconnect();
  });
  observer.observe(root, { childList: true, subtree: true, characterData: true });
  if (markReady()) observer.disconnect();
}
