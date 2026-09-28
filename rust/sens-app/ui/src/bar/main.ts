import "../styles.css";
import "./bar.css";
import { StrictMode, createElement } from "react";
import { createRoot } from "react-dom/client";
import { useStore } from "zustand";
import { language, languageOf, showLanguage } from "../shared/i18n";
import { lookOf, showLook } from "../shared/look";
import { Bar } from "./Bar";
import { boot } from "./store";

showLanguage(languageOf(window.__SENS_LANGUAGE__));
showLook(lookOf(window.__SENS_LOOK__));
boot();

function Spoken() {
  const current = useStore(language, (s) => s.current);
  return createElement(Bar, { key: current });
}

createRoot(document.getElementById("bar")!).render(createElement(StrictMode, null, createElement(Spoken)));
