import { loadCapabilities } from "../features/capabilities/store";
import { chooseLook } from "../features/look/store";
import { chooseLanguage } from "../features/look/useLanguageChoice";
import { setNotices } from "../features/notify/store";
import { project } from "../features/project/store";
import { openSettings, setResident } from "../features/settings/store";
import { checkUpdates, setAutomatic, updates } from "../features/updates/store";
import type { Language } from "../shared/i18n";
import { look, type Accent, type Mode } from "../shared/look";
import { answers } from "./acts";
import { showView } from "./session";

type Preference = "notify" | "keep_in_tray" | "start_with_windows" | "check_updates";

const PREFERENCES: Record<Preference, (on: boolean) => Promise<void>> = {
  notify: setNotices,
  keep_in_tray: (on) => setResident("keepInTray", on),
  start_with_windows: (on) => setResident("startWithWindows", on),
  check_updates: setAutomatic,
};

const ELSEWHERE = "The person is looking at another session, so Sens left their screen as it was.";

async function refreshCapabilities() {
  if (project.getState().view === "capabilities") await loadCapabilities();
  return "";
}

async function setLook({ mode, accent }: { mode: Mode | null; accent: Accent | null }) {
  const chosen = { ...look.getState().chosen, ...(mode ? { mode } : {}), ...(accent ? { accent } : {}) };
  await chooseLook(chosen);
  return `Sens looks ${chosen.mode} with the ${chosen.accent} accent.`;
}

async function setLanguage({ language }: { language: Language }) {
  await chooseLanguage(language);
  return `Sens speaks ${language} now.`;
}

async function setPreference({ name, on }: { name: Preference; on: boolean }) {
  await PREFERENCES[name](on);
  return `${name.replace(/_/g, " ")} is ${on ? "on" : "off"}.`;
}

function showViewFor({ session, view }: { session: string; view: string }) {
  if (project.getState().session !== session) throw new Error(ELSEWHERE);
  if (view === "settings") openSettings();
  else showView(view === "chat" ? "" : (view as "capabilities" | "artifacts" | "news"));
  return `The ${view} view is on screen.`;
}

async function checkForUpdates() {
  await checkUpdates(true);
  const { latest, current, fault, installable } = updates.getState();
  if (fault) throw new Error(fault);
  if (!latest) return `Sens ${current} is the latest version.`;
  return `Sens ${latest.version} is out (this is ${current}). ${installable ? "The person can install it from the pill in the title bar." : "It has to be installed by hand from the website."}`;
}

export function answerPreferences() {
  answers("refresh_capabilities", refreshCapabilities);
  answers("set_look", setLook);
  answers("set_language", setLanguage);
  answers("set_preference", setPreference);
  answers("show_view", showViewFor);
  answers("check_updates", checkForUpdates);
}
