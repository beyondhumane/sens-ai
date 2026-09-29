use sens_agent::said;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::{data_dir, life, profile};

const SPACE: &str = "Space";
const VK_SPACE: u32 = 0x20;
const VK_BEFORE_F1: u32 = 0x6F;
const HIGHEST_F: u32 = 24;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Keys {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub win: bool,
    pub key: String,
}

impl Default for Keys {
    fn default() -> Self {
        Self { ctrl: false, alt: true, shift: false, win: false, key: SPACE.into() }
    }
}

impl Keys {
    pub fn virtual_key(&self) -> Option<u32> {
        let key = self.key.as_str();
        if key == SPACE {
            return Some(VK_SPACE);
        }
        let mut letters = key.chars();
        if let (Some(only), None) = (letters.next(), letters.next()) {
            return (only.is_ascii_uppercase() || only.is_ascii_digit()).then_some(only as u32);
        }
        let number: u32 = key.strip_prefix('F')?.parse().ok()?;
        (1..=HIGHEST_F).contains(&number).then_some(VK_BEFORE_F1 + number)
    }

    pub fn usable(&self) -> bool {
        (self.ctrl || self.alt || self.win) && self.virtual_key().is_some()
    }

    pub fn named(&self) -> String {
        let ctrl = said!(en: "Ctrl", es: "Ctrl", fr: "Ctrl", de: "Strg", ja: "Ctrl", zh: "Ctrl");
        let shift = said!(en: "Shift", es: "Mayús", fr: "Maj", de: "Umschalt", ja: "Shift", zh: "Shift");
        let space = said!(en: "Space", es: "Espacio", fr: "Espace", de: "Leertaste", ja: "Space", zh: "空格");
        let key = if self.key == SPACE { space } else { self.key.clone() };
        [(self.ctrl, ctrl), (self.alt, "Alt".into()), (self.shift, shift), (self.win, "Win".into()), (true, key)]
            .into_iter()
            .filter_map(|(held, name)| held.then_some(name))
            .collect::<Vec<_>>()
            .join("+")
    }
}

pub fn unusable() -> String {
    said!(
        en: "Use Ctrl, Alt or Win with a letter, a number, F1–F24 or Space",
        es: "Usa Ctrl, Alt o Win con una letra, un número, F1–F24 o Espacio",
        fr: "Utilisez Ctrl, Alt ou Win avec une lettre, un chiffre, F1–F24 ou Espace",
        de: "Nutze Strg, Alt oder Win mit einem Buchstaben, einer Zahl, F1–F24 oder der Leertaste",
        ja: "Ctrl、Alt、Win のいずれかと、英字・数字・F1–F24・Space を組み合わせてください",
        zh: "请用 Ctrl、Alt 或 Win 搭配字母、数字、F1–F24 或空格",
    )
}

pub fn taken(keys: &Keys) -> String {
    said!(
        en: "Another app already uses {keys}; Sens keeps the one it had",
        es: "Otra app ya usa {keys}; Sens se queda con el que tenía",
        fr: "Une autre app utilise déjà {keys} ; Sens garde le raccourci qu’il avait",
        de: "Eine andere App nutzt {keys} bereits; Sens behält das bisherige Kürzel",
        ja: "{keys} はほかのアプリが使っています。Sens は元のショートカットのままです",
        zh: "{keys} 已被其他应用占用；Sens 保留原来的快捷键",
        keys = keys.named(),
    )
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct State {
    keys: Keys,
    named: String,
    taken: bool,
}

fn state() -> State {
    let keys = life::shortcut();
    State { named: keys.named(), taken: life::shortcut_taken(), keys }
}

#[tauri::command]
pub fn shortcut_state() -> State {
    state()
}

#[tauri::command(async)]
pub fn shortcut_set(app: AppHandle, keys: Keys) -> Result<State, String> {
    if !keys.usable() {
        return Err(unusable());
    }
    if !life::rebind(life::Wish::Bind(keys.clone())) {
        return Err(taken(&keys));
    }
    profile::set_shortcut(&data_dir(&app)?, keys)?;
    life::retitle(&app);
    Ok(state())
}

#[tauri::command(async)]
pub fn shortcut_pause(paused: bool) {
    life::rebind(if paused { life::Wish::Pause } else { life::Wish::Resume });
}

#[cfg(test)]
mod tests {
    use sens_agent::language::{Language, speaking};

    use super::*;

    fn keys(ctrl: bool, alt: bool, shift: bool, win: bool, key: &str) -> Keys {
        Keys { ctrl, alt, shift, win, key: key.into() }
    }

    #[test]
    fn sens_opens_with_alt_and_space_until_it_is_changed() {
        assert_eq!(Keys::default(), keys(false, true, false, false, "Space"));
        assert_eq!(serde_json::from_str::<Keys>("{}").unwrap(), Keys::default());
    }

    #[test]
    fn letters_numbers_function_keys_and_space_reach_windows_as_their_key_codes() {
        assert_eq!(keys(false, true, false, false, "Space").virtual_key(), Some(0x20));
        assert_eq!(keys(true, false, false, false, "K").virtual_key(), Some(0x4B));
        assert_eq!(keys(true, false, false, false, "7").virtual_key(), Some(0x37));
        assert_eq!(keys(true, false, false, false, "F1").virtual_key(), Some(0x70));
        assert_eq!(keys(true, false, false, false, "F24").virtual_key(), Some(0x87));
        assert_eq!(keys(true, false, false, false, "F25").virtual_key(), None);
        assert_eq!(keys(true, false, false, false, "k").virtual_key(), None);
        assert_eq!(keys(true, false, false, false, "Enter").virtual_key(), None);
    }

    #[test]
    fn a_shortcut_needs_ctrl_alt_or_win_so_typing_never_opens_sens() {
        assert!(keys(false, true, false, false, "Space").usable());
        assert!(keys(false, false, true, true, "S").usable());
        assert!(!keys(false, false, true, false, "S").usable());
        assert!(!keys(false, false, false, false, "F5").usable());
        assert!(!keys(true, true, false, false, "Tab").usable());
    }

    #[test]
    fn the_shortcut_is_named_in_the_language_spoken() {
        let every = keys(true, true, true, true, "Space");
        assert_eq!(speaking(Language::Es, || every.named()), "Ctrl+Alt+Mayús+Win+Espacio");
        assert_eq!(speaking(Language::De, || every.named()), "Strg+Alt+Umschalt+Win+Leertaste");
        assert_eq!(speaking(Language::En, || Keys::default().named()), "Alt+Space");
        assert_eq!(speaking(Language::Es, || keys(true, false, false, false, "F9").named()), "Ctrl+F9");
    }
}
