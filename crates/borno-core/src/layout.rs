//! Keyboard layout definitions, key mappings, and layout configuration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Active Input Mode for the engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InputMode {
    /// Passthrough English (no transformation).
    English,
    /// Borno Layout targeting standard Unicode Bengali output.
    BornoUnicode,
    /// National / Jatiya Layout targeting standard Unicode.
    Jatiya,
    /// Avro-style English-to-Bengali Phonetic Transliteration.
    Phonetic,
}

impl Default for InputMode {
    fn default() -> Self {
        InputMode::English
    }
}

impl InputMode {
    /// Display name of the mode.
    pub fn name(&self) -> &'static str {
        match self {
            InputMode::English => "English",
            InputMode::BornoUnicode => "Borno Unicode",
            InputMode::Jatiya => "National (Jatiya)",
            InputMode::Phonetic => "Avro Phonetic",
        }
    }

    /// Short indicator badge text for UI status pill (e.g. "EN", "বর্ণ", "ক্লাসিক", "ফো").
    pub fn badge(&self) -> &'static str {
        match self {
            InputMode::English => "EN",
            InputMode::BornoUnicode => "বর্ণ",
            InputMode::Jatiya => "জাতীয়",
            InputMode::Phonetic => "ফো",
        }
    }
}

/// A mapping for an individual key under Normal and Shift states.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeyMapping {
    /// Output produced without Shift.
    pub normal: String,
    /// Output produced with Shift held down.
    pub shift: String,
    /// Optional output for AltGr / Ctrl+Alt modifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub altgr: Option<String>,
}

/// Hotkey shortcut combination to switch to this layout.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LayoutHotkey {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    /// ASCII character or virtual key identifier (e.g. 'B', 'V', 'E').
    pub key: char,
}

/// Complete representation of an input keyboard layout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardLayout {
    pub name: String,
    pub mode: InputMode,
    pub hotkey: Option<LayoutHotkey>,
    /// Map of physical keyboard character (e.g. 'j', 'k', '1') to KeyMapping.
    pub keys: HashMap<char, KeyMapping>,
    /// Sequential conjunct substitution rules.
    #[serde(default)]
    pub conjuncts: HashMap<String, String>,
}

impl KeyboardLayout {
    pub fn new(name: impl Into<String>, mode: InputMode) -> Self {
        Self {
            name: name.into(),
            mode,
            hotkey: None,
            keys: HashMap::new(),
            conjuncts: HashMap::new(),
        }
    }

    pub fn with_hotkey(mut self, ctrl: bool, alt: bool, shift: bool, key: char) -> Self {
        self.hotkey = Some(LayoutHotkey {
            ctrl,
            alt,
            shift,
            key: key.to_ascii_uppercase(),
        });
        self
    }

    pub fn add_key(&mut self, physical_key: char, normal: impl Into<String>, shift: impl Into<String>) {
        self.keys.insert(
            physical_key,
            KeyMapping {
                normal: normal.into(),
                shift: shift.into(),
                altgr: None,
            },
        );
    }

    pub fn add_conjunct(&mut self, sequence: impl Into<String>, replacement: impl Into<String>) {
        self.conjuncts.insert(sequence.into(), replacement.into());
    }

    /// Look up the output for a physical key.
    pub fn lookup(&self, key: char, shift: bool) -> Option<&str> {
        self.keys.get(&key).map(|km| {
            if shift {
                km.shift.as_str()
            } else {
                km.normal.as_str()
            }
        })
    }
}
