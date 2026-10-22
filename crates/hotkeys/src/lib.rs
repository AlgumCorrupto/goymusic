use std::{fmt, str::FromStr, sync::Arc};

#[cfg(feature = "native")]
mod native;
#[cfg(all(feature = "portal", target_os = "linux"))]
mod portal;

#[cfg(feature = "native")]
pub use native::NativeHotkeys;
#[cfg(all(feature = "portal", target_os = "linux"))]
pub use portal::PortalHotkeys;

/// Called from backend threads when a hotkey fires while the UI is idle
/// (e.g. `move || ctx.request_repaint()`).
pub type Wake = Arc<dyn Fn() + Send + Sync>;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct Modifiers {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub meta: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Key {
    Char(char), // lowercase 'a'..='z' or '0'..='9'
    F(u8),      // 1..=12
    Space,
    Enter,
    Escape,
    Tab,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Hotkey {
    pub mods: Modifiers,
    pub key: Key,
}

impl FromStr for Hotkey {
    type Err = String;

    /// "Alt+T", "ctrl+shift+F5", "Meta+Space"
    fn from_str(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.split('+').map(str::trim).collect();
        let (key_str, mod_strs) = parts.split_last().ok_or("empty hotkey")?;

        let mut mods = Modifiers::default();
        for m in mod_strs {
            match m.to_ascii_lowercase().as_str() {
                "ctrl" | "control" => mods.ctrl = true,
                "shift" => mods.shift = true,
                "alt" => mods.alt = true,
                "meta" | "super" | "logo" | "win" | "cmd" => mods.meta = true,
                other => return Err(format!("unknown modifier '{other}'")),
            }
        }

        let k = key_str.to_ascii_lowercase();
        let key = match k.as_str() {
            "space" => Key::Space,
            "enter" | "return" => Key::Enter,
            "esc" | "escape" => Key::Escape,
            "tab" => Key::Tab,
            f if f.starts_with('f') && f.len() > 1 => match f[1..].parse::<u8>() {
                Ok(n) if (1..=12).contains(&n) => Key::F(n),
                _ => return Err(format!("unknown key '{key_str}'")),
            },
            c if c.len() == 1 && c.chars().all(|ch| ch.is_ascii_alphanumeric()) => {
                Key::Char(c.chars().next().unwrap())
            }
            _ => return Err(format!("unknown key '{key_str}'")),
        };
        Ok(Hotkey { mods, key })
    }
}

impl fmt::Display for Hotkey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.mods.ctrl { write!(f, "Ctrl+")?; }
        if self.mods.shift { write!(f, "Shift+")?; }
        if self.mods.alt { write!(f, "Alt+")?; }
        if self.mods.meta { write!(f, "Meta+")?; }
        match self.key {
            Key::Char(c) => write!(f, "{}", c.to_ascii_uppercase()),
            Key::F(n) => write!(f, "F{n}"),
            Key::Space => write!(f, "Space"),
            Key::Enter => write!(f, "Enter"),
            Key::Escape => write!(f, "Escape"),
            Key::Tab => write!(f, "Tab"),
        }
    }
}

/// One global shortcut your app wants. Both backends take the same description.
#[derive(Clone)]
pub struct Binding<A> {
    pub action: A,
    pub id: &'static str,          // stable id, used by the portal ("quick-tts")
    pub description: &'static str, // shown in KDE's shortcut settings
    pub trigger: Hotkey,           // requested key (a preference on the portal)
}

pub trait GlobalHotkeys<A> {
    fn backend_name(&self) -> &'static str;
    /// Non-blocking: actions fired since the last call.
    fn poll(&mut self) -> Vec<A>;
}

/// Accepts everything, fires nothing.
pub struct DummyHotkeys;

impl<A> GlobalHotkeys<A> for DummyHotkeys {
    fn backend_name(&self) -> &'static str { "none" }
    fn poll(&mut self) -> Vec<A> { vec![] }
}

/// Picks a backend: the portal on Wayland, `global-hotkey` elsewhere,
/// the dummy if nothing works.
#[allow(unused_variables)]
pub fn connect<A: Copy + Send + 'static>(
    bindings: &[Binding<A>],
    wake: Wake,
) -> Box<dyn GlobalHotkeys<A>> {
    #[cfg(all(feature = "portal", target_os = "linux"))]
    if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        // global-hotkey can't work on Wayland, so don't fall back to it here.
        return match PortalHotkeys::new(bindings, wake) {
            Ok(p) => Box::new(p),
            Err(e) => {
                eprintln!("hotkeys: portal unavailable: {e}");
                Box::new(DummyHotkeys)
            }
        };
    }

    #[cfg(feature = "native")]
    match NativeHotkeys::new(bindings, wake) {
        Ok(n) => return Box::new(n),
        Err(e) => eprintln!("hotkeys: native backend unavailable: {e}"),
    }

    Box::new(DummyHotkeys)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_display_roundtrip() {
        let hk: Hotkey = "ctrl+shift+f5".parse().unwrap();
        assert_eq!(hk.to_string(), "Ctrl+Shift+F5");
        assert!("Alt+".parse::<Hotkey>().is_err());
    }
}