import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import "./styles.css";
import "./fixture-modes.css";

const root = document.getElementById("root");

if (!root) {
  throw new Error("Winds desktop root element is missing");
}

const params = new URLSearchParams(window.location.search);
const documentRoot = document.documentElement;
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
