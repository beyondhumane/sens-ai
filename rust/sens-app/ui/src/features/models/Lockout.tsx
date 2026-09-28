import { useStore } from "zustand";
import { Callout } from "../../shared/Callout";
import { Icon } from "../../shared/Icon";
import { ICONS } from "../../shared/icons.js";
import { openSettings, settings, signInAgain } from "../settings/store";
import { t } from "./copy";
import { lockoutShown, models } from "./store";

export function LockoutNotice() {
  useStore(models, (s) => s.lockout);
  useStore(models, (s) => s.account);
  const connecting = useStore(settings, (s) => s.connecting);
  const shown = lockoutShown();
  if (!shown) return null;
  const toProviders = shown === "key" || shown === "credit";

  return (
    <Callout icon={ICONS.shieldAlert} title={t.lockout[shown]} said={t.lockoutSaid[shown]} role="alert">
      {toProviders ? (
        <button className="primary" onClick={(event) => openSettings("providers", event.currentTarget)}>
          {t.providers}
        </button>
      ) : (
        <button className="primary" disabled={connecting} onClick={(event) => signInAgain(event.currentTarget)}>
          <Icon svg={ICONS.logIn} />
          {connecting ? t.signingIn : t.signIn}
        </button>
      )}
    </Callout>
  );
}
