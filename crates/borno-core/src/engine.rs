//! Core typing engine state machine and event dispatcher.

use crate::unicode::create_borno_unicode_layout;
use crate::layout::{InputMode, KeyboardLayout};
use crate::phonetic::PhoneticEngine;

/// Action emitted by the engine to the OS input driver.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineAction {
    /// Let the operating system handle the physical key untouched.
    Passthrough,
    /// Send replacement text directly without erasing anything.
    Emit(String),
    /// Erase `backspaces` characters and inject `text`.
    Replace {
        backspaces: usize,
        text: String,
    },
    /// Layout mode was switched (e.g. from English to Borno Unicode).
    ModeSwitch(InputMode),
}

/// The core BornoFree input engine.
pub struct BornoEngine {
    pub mode: InputMode,
    prev_bangla_mode: InputMode,
    borno_unicode: KeyboardLayout,
    phonetic: PhoneticEngine,
    buffer: String,
    pub pending_pre_kar: Option<char>,
    pub pending_bindu: Option<char>,
}

impl Default for BornoEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl BornoEngine {
    pub fn new() -> Self {
        Self {
            mode: InputMode::English,
            prev_bangla_mode: InputMode::BornoUnicode,
            borno_unicode: create_borno_unicode_layout(),
            phonetic: PhoneticEngine::new(),
            buffer: String::with_capacity(64),
            pending_pre_kar: None,
            pending_bindu: None,
        }
    }

    /// Set the current input mode.
    pub fn set_mode(&mut self, new_mode: InputMode) {
        if new_mode != InputMode::English {
            self.prev_bangla_mode = new_mode;
        }
        self.mode = new_mode;
        self.buffer.clear();
        self.pending_pre_kar = None;
        self.pending_bindu = None;
    }

    /// Returns the current text buffer.
    pub fn get_buffer(&self) -> &str {
        &self.buffer
    }

    /// Toggle between English and the most recently used Bengali mode (F12 shortcut).
    pub fn toggle_mode(&mut self) -> InputMode {
        let target = if self.mode == InputMode::English {
            self.prev_bangla_mode
        } else {
            InputMode::English
        };
        self.set_mode(target);
        target
    }

    /// Process a virtual key code directly (useful for OS-level hooks).
    pub fn handle_vk(&mut self, vk: u32, shift: bool, ctrl: bool, alt: bool) -> EngineAction {
        // Hotkey: F12 (VK_F12 = 0x7B) quick toggle between English and Bengali
        if vk == 0x7B {
            let new_mode = self.toggle_mode();
            return EngineAction::ModeSwitch(new_mode);
        }

        if let Some(ch) = vk_to_char(vk) {
            self.handle_key(ch, shift, ctrl, alt)
        } else {
            EngineAction::Passthrough
        }
    }

    /// Process a raw keyboard event.
    ///
    /// - `key`: Character or virtual key representation.
    /// - `shift`: Whether Shift is held.
    /// - `ctrl`: Whether Ctrl is held.
    /// - `alt`: Whether Alt is held.
    pub fn handle_key(&mut self, key: char, shift: bool, ctrl: bool, alt: bool) -> EngineAction {
        // 1. Check Hotkey Switches (Borno Toggle Behavior)
        if ctrl && alt {
            match key.to_ascii_uppercase() {
                'V' => {
                    let target_mode = if self.mode == InputMode::BornoUnicode {
                        InputMode::English
                    } else {
                        InputMode::BornoUnicode
                    };
                    self.set_mode(target_mode);
                    return EngineAction::ModeSwitch(target_mode);
                }
                'E' => {
                    self.set_mode(InputMode::English);
                    return EngineAction::ModeSwitch(InputMode::English);
                }
                _ => {}
            }
        }

        // If Ctrl or Alt is pressed (e.g. Ctrl+C, Ctrl+V, Alt+Tab), pass through.
        if ctrl || alt {
            return EngineAction::Passthrough;
        }

        // 2. English Mode: Passthrough all
        if self.mode == InputMode::English {
            return EngineAction::Passthrough;
        }

        // 3. Avro Phonetic Mode
        if self.mode == InputMode::Phonetic {
            if key == ' ' || key == '\n' || key == '\t' {
                // Word boundary: commit phonetic transliteration
                if !self.buffer.is_empty() {
                    let out = self.phonetic.transliterate(&self.buffer);
                    let backspaces = self.buffer.len();
                    self.buffer.clear();
                    return EngineAction::Replace {
                        backspaces,
                        text: format!("{}{}", out, key),
                    };
                }
                return EngineAction::Passthrough;
            } else if key == '\x08' {
                // Backspace
                self.buffer.pop();
                return EngineAction::Passthrough;
            } else if key.is_ascii_alphabetic() {
                self.buffer.push(key);
                return EngineAction::Passthrough;
            }
        }

        // 4. Borno Unicode Mode (with Pre-Kar Deferred Reordering)
        if self.mode == InputMode::BornoUnicode {
            self.handle_borno_unicode_key(key, shift)
        } else {
            EngineAction::Passthrough
        }
    }

    /// Handles keystrokes in Borno Unicode mode with intelligent pre-kar deferred reordering.
    ///
    /// Typists press pre-kars (ি, ে, ৈ) BEFORE the consonant.
    /// Standard Unicode requires the consonant FIRST, then the vowel sign (Kar) SECOND.
    /// This engine holds pre-kars and emits them in correct Unicode logical order.
    fn handle_borno_unicode_key(&mut self, key: char, shift: bool) -> EngineAction {
        // Backspace handling
        if key == '\x08' {
            if self.pending_bindu.is_some() {
                self.pending_bindu = None;
                return EngineAction::Replace {
                    backspaces: 0,
                    text: String::new(),
                };
            }
            if let Some(kar) = self.pending_pre_kar.take() {
                if self.buffer.ends_with('্') {
                    // Case 3 had replaced pre-kar with '্' on screen.
                    // User hit backspace to undo 'g' ('্').
                    // Erase '্' and restore the pre-kar to screen & buffer!
                    self.buffer.pop();
                    self.buffer.push(kar);
                    return EngineAction::Replace {
                        backspaces: 1,
                        text: kar.to_string(),
                    };
                }
                return EngineAction::Replace {
                    backspaces: 0,
                    text: String::new(),
                };
            }
            self.buffer.pop();
            return EngineAction::Passthrough;
        }

        let layout = &self.borno_unicode;
        let mapped = if key == 'a' && shift {
            let last_char = self.buffer.chars().last();
            if last_char.is_none() || last_char.map(|c| c.is_whitespace() || c == '।').unwrap_or(false) {
                "ঋ"
            } else {
                "র্"
            }
        } else {
            match layout.lookup(key, shift) {
                Some(m) => m,
                None => {
                    // Unmapped key (Space, Enter, Tab, etc.)
                    if let Some(kar) = self.pending_pre_kar.take() {
                        let bindu = self.pending_bindu.take().map(|c| c.to_string()).unwrap_or_default();
                        if self.buffer.ends_with('্') {
                            self.buffer.pop();
                            let text = format!("{}{}{}", kar, bindu, key);
                            self.buffer.push_str(&text);
                            return EngineAction::Replace {
                                backspaces: 1,
                                text,
                            };
                        }
                        self.buffer.clear();
                        return EngineAction::Emit(format!("{}{}{}", kar, bindu, key));
                    }
                    self.pending_bindu = None;
                    self.buffer.clear();
                    return EngineAction::Passthrough;
                }
            }
        };

        // Case 1: Check if previous char in buffer is Hasanta ('্')
        // Typing 'g' ('্') followed by a kar forms a full vowel (e.g. '্' + 'ি' = 'ই', '্' + 'া' = 'আ')
        if self.buffer.ends_with('্') {
            let probe = format!("{}{}", self.buffer, mapped);
            for (seq, repl) in &layout.conjuncts {
                if probe.ends_with(seq) {
                    let backspaces = seq.chars().count() - 1;
                    let prefix_len = probe.len() - seq.len();
                    let bindu_str = self.pending_bindu.take().map(|c| c.to_string()).unwrap_or_default();
                    let (text, new_buf) = if let Some(saved_kar) = self.pending_pre_kar.take() {
                        (
                            format!("{}{}{}", saved_kar, bindu_str, repl),
                            format!("{}{}{}{}", &probe[..prefix_len], saved_kar, bindu_str, repl),
                        )
                    } else {
                        (
                            repl.clone(),
                            format!("{}{}", &probe[..prefix_len], repl),
                        )
                    };
                    self.buffer = new_buf;
                    return EngineAction::Replace {
                        backspaces,
                        text,
                    };
                }
            }
        }

        // Case 2: The current key produces a Pre-Kar ('ি', 'ে', 'ৈ')
        let first_char = mapped.chars().next().unwrap_or(' ');
        if mapped.chars().count() == 1 && crate::bengali::is_pre_kar(first_char) {
            if let Some(prev_kar) = self.pending_pre_kar.take() {
                // If a pre-kar was already held, emit the previous one and hold the new one
                let bindu = self.pending_bindu.take().map(|c| c.to_string()).unwrap_or_default();
                self.pending_pre_kar = Some(first_char);
                let to_emit = format!("{}{}", prev_kar, bindu);
                self.buffer.push_str(&to_emit);
                return EngineAction::Emit(to_emit);
            } else {
                // Hold the pre-kar! Eat the physical keystroke (nothing emitted yet).
                self.pending_pre_kar = Some(first_char);
                return EngineAction::Replace {
                    backspaces: 0,
                    text: String::new(),
                };
            }
        }

        // Case 3: The current key produces Hasanta ('্') or a Phala ('্র', '্য') while buffer ends with Pre-Kar
        if self.buffer.ends_with(|c| crate::bengali::is_pre_kar(c)) {
            if mapped == "্" {
                // Single Hasanta ('্'): extends consonant cluster (e.g. 'শক্তি' where typist typed 'ি' + 'ক' + '্' + 'ত')
                if let Some(last_kar) = self.buffer.pop() {
                    self.pending_pre_kar = Some(last_kar);
                    self.buffer.push('্');
                    return EngineAction::Replace {
                        backspaces: 1,
                        text: "্".to_string(),
                    };
                }
            } else if mapped.starts_with('্') {
                // Compound Phala (e.g. '্র' = '্' + 'র', '্য' = '্' + 'য'):
                // Pre-kar was already attached to base consonant, now user adds R-phala / Ya-phala (e.g. 'তি' + '্র' -> 'ত্রি', 'পে' + '্র' -> 'প্রে').
                // Invert the order: move pre-kar after the phala!
                if let Some(last_kar) = self.buffer.pop() {
                    let bindu_str = self.pending_bindu.take().map(|c| c.to_string()).unwrap_or_default();
                    let text = format!("{}{}{}", mapped, last_kar, bindu_str);
                    self.buffer.push_str(&text);
                    return EngineAction::Replace {
                        backspaces: 1,
                        text,
                    };
                }
            }
        }

        // Case 4: A Pre-Kar is currently held, and user typed a Consonant or combining sign
        if let Some(kar) = self.pending_pre_kar {
            // Check if user entered a combining mark (Chandrabindu 'ঁ', etc.) while pre-kar is held
            if mapped == "ঁ" || (mapped.chars().count() == 1 && crate::bengali::is_combining_mark(first_char)) {
                self.pending_bindu = Some(first_char);
                return EngineAction::Replace {
                    backspaces: 0,
                    text: String::new(),
                };
            }

            // If user typed Reph ('র্' = 'র' + '্') while a pre-kar is held,
            // the pre-kar belongs to the subsequent base consonant (e.g. in 'ঊর্মি', 'ি' belongs to 'ম', not 'র্').
            // Emit 'র্', add to buffer, but KEEP pending_pre_kar held!
            if mapped == "র্" {
                self.buffer.push_str("র্");
                return EngineAction::Emit("র্".to_string());
            }

            self.pending_pre_kar = None;
            let bindu = self.pending_bindu.take();
            let first_ch = mapped.chars().next().unwrap_or(' ');
            if crate::bengali::is_consonant(first_ch) {
                let combined = if let Some(b) = bindu {
                    format!("{}{}{}", mapped, kar, b)
                } else {
                    format!("{}{}", mapped, kar)
                };
                self.buffer.push_str(&combined);
                return EngineAction::Emit(combined);
            } else {
                let bindu_str = bindu.map(|c| c.to_string()).unwrap_or_default();
                if self.buffer.ends_with('্') {
                    self.buffer.pop();
                    let text = format!("{}{}{}", kar, bindu_str, mapped);
                    self.buffer.push_str(&text);
                    return EngineAction::Replace {
                        backspaces: 1,
                        text,
                    };
                } else {
                    let combined = format!("{}{}{}", kar, bindu_str, mapped);
                    // Check if combined matches any conjunct rule (e.g. 'ে' + 'া' -> 'ো', 'ে' + 'ৗ' -> 'ৌ')
                    let mut matched_repl = None;
                    for (seq, repl) in &layout.conjuncts {
                        if combined.ends_with(seq) {
                            matched_repl = Some((seq.clone(), repl.clone()));
                            break;
                        }
                    }
                    if let Some((seq, repl)) = matched_repl {
                        let prefix = &combined[..combined.len() - seq.len()];
                        let text = format!("{}{}", prefix, repl);
                        self.buffer.push_str(&text);
                        return EngineAction::Emit(text);
                    } else {
                        self.buffer.push_str(&combined);
                        return EngineAction::Emit(combined);
                    }
                }
            }
        }

        // Case 5: No pending pre-kar. Check conjunct replacements (e.g. 'ে' + 'া' -> 'ো')
        let probe = format!("{}{}", self.buffer, mapped);
        for (seq, repl) in &layout.conjuncts {
            if probe.ends_with(seq) {
                let backspaces = seq.chars().count() - 1;
                let prefix_len = probe.len() - seq.len();
                self.buffer = format!("{}{}", &probe[..prefix_len], repl);
                return EngineAction::Replace {
                    backspaces,
                    text: repl.clone(),
                };
            }
        }

        self.buffer.push_str(mapped);
        if self.buffer.chars().count() > 32 {
            let start = self.buffer.char_indices().nth_back(16).map(|(i, _)| i).unwrap_or(0);
            self.buffer = self.buffer[start..].to_string();
        }
        EngineAction::Emit(mapped.to_string())
    }
}

/// Helper to map common Windows Virtual Key Codes (VK) to their base lowercase characters.
pub fn vk_to_char(vk: u32) -> Option<char> {
    match vk {
        0x30..=0x39 => char::from_u32(vk), // '0'..'9'
        0x41..=0x5A => char::from_u32(vk + 32), // 'A'..'Z' -> 'a'..'z'
        0xBA => Some(';'),  // VK_OEM_1
        0xBB => Some('='),  // VK_OEM_PLUS
        0xBC => Some(','),  // VK_OEM_COMMA
        0xBD => Some('-'),  // VK_OEM_MINUS
        0xBE => Some('.'),  // VK_OEM_PERIOD
        0xBF => Some('/'),  // VK_OEM_2
        0xC0 => Some('`'),  // VK_OEM_3
        0xDB => Some('['),  // VK_OEM_4
        0xDC => Some('\\'), // VK_OEM_5
        0xDD => Some(']'),  // VK_OEM_6
        0xDE => Some('\''), // VK_OEM_7
        0x20 => Some(' '),  // VK_SPACE
        0x08 => Some('\x08'), // VK_BACK
        0x0D => Some('\n'), // VK_RETURN
        _ => None,
    }
}
