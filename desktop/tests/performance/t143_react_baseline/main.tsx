import { StrictMode, useEffect } from "react";
import { createRoot } from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";

function Baseline() {
  useEffect(() => {
    window.requestAnimationFrame(() => {
      window.requestAnimationFrame(() => {
        void getCurrentWindow().setTitle("Winds [T143 React Baseline]");
      });
    });
  }, []);
  return <main>Winds T143 minimal React baseline</main>;
}

const root = document.getElementById("root");
if (!root) throw new Error("T143 React baseline root missing");
createRoot(root).render(<StrictMode><Baseline /></StrictMode>);
