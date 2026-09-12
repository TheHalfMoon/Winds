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
const theme = params.get("theme");

if (theme === "dark" || theme === "light" || theme === "contrast") {
  document.documentElement.dataset.theme = theme;
} else {
  document.documentElement.dataset.theme = "dark";
}

if (params.get("fixture") === "keyboard-focus") {
  document.documentElement.dataset.fixture = "keyboard-focus";
}

if (params.get("scale") === "125") {
  document.documentElement.dataset.scale = "125";
}

createRoot(root).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
