import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import "./app.css";
import { readClosureClassFromMiddle } from "./hmr-closure/middle";

Object.assign(window, { __readOmenaHmrClosure: readClosureClassFromMiddle });

const container = document.getElementById("root");
if (!container) throw new Error("no #root");

createRoot(container).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
