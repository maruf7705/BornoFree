//! # BornoFree Core (`borno-core`)
//!
//! High-performance, cross-platform Bengali typing engine, authentic layouts,
//! and typing engine.

pub mod bengali;
pub mod unicode;
pub mod engine;
pub mod layout;
pub mod phonetic;

pub use bengali::*;
pub use unicode::*;
pub use engine::*;
pub use layout::*;
pub use phonetic::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_borno_unicode_typing() {
        let mut engine = BornoEngine::new();
        engine.set_mode(InputMode::BornoUnicode);

        // 'j' should produce 'ক'
        assert_eq!(
            engine.handle_key('j', false, false, false),
            EngineAction::Emit("ক".to_string())
        );

        // Shift + 'j' should produce 'খ'
        assert_eq!(
            engine.handle_key('j', true, false, false),
            EngineAction::Emit("খ".to_string())
        );

        // Vowel combination: 'g' (্) + 'f' (া) -> আ
        assert_eq!(
            engine.handle_key('g', false, false, false),
            EngineAction::Emit("্".to_string())
        );
        assert_eq!(
            engine.handle_key('f', false, false, false),
            EngineAction::Replace {
                backspaces: 1,
                text: "আ".to_string()
            }
        );
    }

    #[test]
    fn test_pre_kar_reordering() {
        let mut engine = BornoEngine::new();
        engine.set_mode(InputMode::BornoUnicode);

        // 1. Test "কি" (typing Pre-Kar 'd' (ি) before 'j' (ক))
        assert_eq!(
            engine.handle_key('d', false, false, false),
            EngineAction::Replace {
                backspaces: 0,
                text: "".to_string()
            }
        );
        assert_eq!(
            engine.handle_key('j', false, false, false),
            EngineAction::Emit("কি".to_string())
        );

        // 2. Test "কে" (typing Pre-Kar 'c' (ে) before 'j' (ক))
        assert_eq!(
            engine.handle_key('c', false, false, false),
            EngineAction::Replace {
                backspaces: 0,
                text: "".to_string()
            }
        );
        assert_eq!(
            engine.handle_key('j', false, false, false),
            EngineAction::Emit("কে".to_string())
        );

        // 3. Test "রি" (typing Pre-Kar 'd' (ি) before 'v' (র))
        assert_eq!(
            engine.handle_key('d', false, false, false),
            EngineAction::Replace {
                backspaces: 0,
                text: "".to_string()
            }
        );
        assert_eq!(
            engine.handle_key('v', false, false, false),
            EngineAction::Emit("রি".to_string())
        );

        // 4. Test "মি" in "আমি" (typing Pre-Kar 'd' (ি) before 'm' (ম))
        assert_eq!(
            engine.handle_key('d', false, false, false),
            EngineAction::Replace {
                backspaces: 0,
                text: "".to_string()
            }
        );
        assert_eq!(
            engine.handle_key('m', false, false, false),
            EngineAction::Emit("মি".to_string())
        );

        // 5. Test "লো" (typing 'c' (ে) + Shift+'v' (ল) -> "লে", then 'f' (া) -> "লো")
        assert_eq!(
            engine.handle_key('c', false, false, false),
            EngineAction::Replace {
                backspaces: 0,
                text: "".to_string()
            }
        );
        assert_eq!(
            engine.handle_key('v', true, false, false),
            EngineAction::Emit("লে".to_string())
        );
        assert_eq!(
            engine.handle_key('f', false, false, false),
            EngineAction::Replace {
                backspaces: 1,
                text: "ো".to_string()
            }
        );
    }

    #[test]
    fn test_dirgha_ii_and_i_vowels() {
        let mut engine = BornoEngine::new();
        engine.set_mode(InputMode::BornoUnicode);

        // 1. 'g' (্) + 'd' (ি) -> independent 'ই'
        assert_eq!(
            engine.handle_key('g', false, false, false),
            EngineAction::Emit("্".to_string())
        );
        assert_eq!(
            engine.handle_key('d', false, false, false),
            EngineAction::Replace {
                backspaces: 1,
                text: "ই".to_string()
            }
        );

        // 2. 'g' (্) + Shift+'d' (ী) -> independent 'ঈ'
        assert_eq!(
            engine.handle_key('g', false, false, false),
            EngineAction::Emit("্".to_string())
        );
        assert_eq!(
            engine.handle_key('d', true, false, false),
            EngineAction::Replace {
                backspaces: 1,
                text: "ঈ".to_string()
            }
        );

        // 3. 'j' (ক) + Shift+'d' (ী) -> 'কী' (Dirgha I-kar, NOT independent 'ঈ')
        assert_eq!(
            engine.handle_key('j', false, false, false),
            EngineAction::Emit("ক".to_string())
        );
        assert_eq!(
            engine.handle_key('d', true, false, false),
            EngineAction::Emit("ী".to_string())
        );
    }

    #[test]
    fn test_oi_vowel_and_kar() {
        let mut engine = BornoEngine::new();
        engine.set_mode(InputMode::BornoUnicode);

        // 1. Shift+'c' (ৈ) + 'j' (ক) -> 'কৈ'
        assert_eq!(
            engine.handle_key('c', true, false, false),
            EngineAction::Replace {
                backspaces: 0,
                text: "".to_string()
            }
        );
        assert_eq!(
            engine.handle_key('j', false, false, false),
            EngineAction::Emit("কৈ".to_string())
        );

        // Reset between words
        engine.handle_key(' ', false, false, false);

        // 2. 'g' (্) + Shift+'c' (ৈ) -> independent 'ঐ'
        assert_eq!(
            engine.handle_key('g', false, false, false),
            EngineAction::Emit("্".to_string())
        );
        assert_eq!(
            engine.handle_key('c', true, false, false),
            EngineAction::Replace {
                backspaces: 1,
                text: "ঐ".to_string()
            }
        );
    }

    #[test]
    fn test_shakti_cluster_and_eguloi() {
        let mut engine = BornoEngine::new();
        engine.set_mode(InputMode::BornoUnicode);

        // 1. Unicode Layout typing of 'শক্তি': 'শ' (Shift+'m') + 'ি' ('d') + 'ক' ('j') + '্' ('g') + 'ত' ('k') -> 'শক্তি'
        assert_eq!(
            engine.handle_key('m', true, false, false),
            EngineAction::Emit("শ".to_string())
        );
        assert_eq!(
            engine.handle_key('d', false, false, false),
            EngineAction::Replace {
                backspaces: 0,
                text: "".to_string()
            }
        );
        assert_eq!(
            engine.handle_key('j', false, false, false),
            EngineAction::Emit("কি".to_string())
        );
        assert_eq!(
            engine.handle_key('g', false, false, false),
            EngineAction::Replace {
                backspaces: 1,
                text: "্".to_string()
            }
        );
        assert_eq!(
            engine.handle_key('k', false, false, false),
            EngineAction::Emit("তি".to_string())
        );

        // 2. Typing 'এগুলোই':
        // Reset/space
        engine.handle_key(' ', false, false, false);

        // 'এ' = 'g' + 'c'
        engine.handle_key('g', false, false, false);
        assert_eq!(
            engine.handle_key('c', false, false, false),
            EngineAction::Replace {
                backspaces: 1,
                text: "এ".to_string()
            }
        );
        // 'গু' = 'o' + 's'
        assert_eq!(
            engine.handle_key('o', false, false, false),
            EngineAction::Emit("গ".to_string())
        );
        assert_eq!(
            engine.handle_key('s', false, false, false),
            EngineAction::Emit("ু".to_string())
        );
        // 'লো' = 'c' + Shift+'v' ('ল') -> 'লে', then 'f' ('া') -> replaces 'ে' with 'ো'
        engine.handle_key('c', false, false, false);
        assert_eq!(
            engine.handle_key('v', true, false, false),
            EngineAction::Emit("লে".to_string())
        );
        assert_eq!(
            engine.handle_key('f', false, false, false),
            EngineAction::Replace {
                backspaces: 1,
                text: "ো".to_string()
            }
        );
        // 'ই' = 'g' + 'd' (independent vowel at end of 'এগুলো')
        assert_eq!(
            engine.handle_key('g', false, false, false),
            EngineAction::Emit("্".to_string())
        );
        assert_eq!(
            engine.handle_key('d', false, false, false),
            EngineAction::Replace {
                backspaces: 1,
                text: "ই".to_string()
            }
        );
    }

    #[test]
    fn test_phonetic_transliteration() {
        let phonetic = PhoneticEngine::new();
        assert_eq!(phonetic.transliterate("ami"), "আমি");
        assert_eq!(phonetic.transliterate("bangla"), "বাংলা");
    }

    #[test]
    fn test_hotkey_switching() {
        let mut engine = BornoEngine::new();
        assert_eq!(engine.mode, InputMode::English);

        // 1. Press Ctrl + Alt + V -> Switches to BornoUnicode
        assert_eq!(
            engine.handle_key('v', false, true, true),
            EngineAction::ModeSwitch(InputMode::BornoUnicode)
        );
        assert_eq!(engine.mode, InputMode::BornoUnicode);

        // 2. Press Ctrl + Alt + V again -> Toggles back to English!
        assert_eq!(
            engine.handle_key('v', false, true, true),
            EngineAction::ModeSwitch(InputMode::English)
        );
        assert_eq!(engine.mode, InputMode::English);

        // 5. Press Ctrl + Alt + E -> Explicit English switch
        assert_eq!(
            engine.handle_key('e', false, true, true),
            EngineAction::ModeSwitch(InputMode::English)
        );
        assert_eq!(engine.mode, InputMode::English);
    }

    #[test]
    fn test_ri_vocalic_ri_and_khanda_ta() {
        let mut engine = BornoEngine::new();
        engine.set_mode(InputMode::BornoUnicode);

        // 1. Shift+'a' at word start emits independent 'ঋ' (U+098B)
        assert_eq!(
            engine.handle_key('a', true, false, false),
            EngineAction::Emit("ঋ".to_string())
        );

        // Reset
        engine.handle_key(' ', false, false, false);

        // 2. Unicode Layout dead-key: 'g' (্) + 'a' (ৃ) -> 'ঋ'
        assert_eq!(
            engine.handle_key('g', false, false, false),
            EngineAction::Emit("্".to_string())
        );
        assert_eq!(
            engine.handle_key('a', false, false, false),
            EngineAction::Replace {
                backspaces: 1,
                text: "ঋ".to_string()
            }
        );

        // 3. 'a' (unshifted) following a consonant emits 'ৃ' (U+09C3) -> 'কৃ'
        assert_eq!(
            engine.handle_key('j', false, false, false),
            EngineAction::Emit("ক".to_string())
        );
        assert_eq!(
            engine.handle_key('a', false, false, false),
            EngineAction::Emit("ৃ".to_string())
        );

        // 4. Shift+'a' following a consonant emits Reph 'র্' (U+09B0 U+09CD) -> 'কর্ম'
        engine.handle_key(' ', false, false, false);
        assert_eq!(
            engine.handle_key('j', false, false, false),
            EngineAction::Emit("ক".to_string())
        );
        assert_eq!(
            engine.handle_key('a', true, false, false),
            EngineAction::Emit("র্".to_string())
        );
        assert_eq!(
            engine.handle_key('m', false, false, false),
            EngineAction::Emit("ম".to_string())
        );

        // 5. Khanda-Ta via backslash key '\'
        engine.handle_key(' ', false, false, false);
        assert_eq!(
            engine.handle_key('\\', false, false, false),
            EngineAction::Emit("ৎ".to_string())
        );

        // 6. Khanda-Ta via Shift+'g'
        assert_eq!(
            engine.handle_key('g', true, false, false),
            EngineAction::Emit("ৎ".to_string())
        );

        // 7. Visarga via Shift+'\'
        assert_eq!(
            engine.handle_key('\\', true, false, false),
            EngineAction::Emit("ঃ".to_string())
        );

    }

    #[test]
    fn test_pre_kar_with_chandrabindu_and_backspace() {
        let mut engine = BornoEngine::new();
        engine.set_mode(InputMode::BornoUnicode);

        // 1. Typing 'c' (ে) followed by Shift+'7' (ঁ) followed by 'j' (ক) -> "কেঁ"
        assert_eq!(
            engine.handle_key('c', false, false, false),
            EngineAction::Replace {
                backspaces: 0,
                text: "".to_string(),
            }
        );
        assert_eq!(
            engine.handle_key('7', true, false, false),
            EngineAction::Replace {
                backspaces: 0,
                text: "".to_string(),
            }
        );
        assert_eq!(
            engine.handle_key('j', false, false, false),
            EngineAction::Emit("কেঁ".to_string())
        );

        // 2. Typing 'd' (ি), then Backspace cancels the pending pre-kar cleanly without emitting
        engine.handle_key(' ', false, false, false);
        assert_eq!(
            engine.handle_key('d', false, false, false),
            EngineAction::Replace {
                backspaces: 0,
                text: "".to_string(),
            }
        );
        assert_eq!(
            engine.handle_key('\x08', false, false, false),
            EngineAction::Replace {
                backspaces: 0,
                text: "".to_string(),
            }
        );
        assert_eq!(engine.pending_pre_kar, None);
        assert_eq!(engine.pending_bindu, None);
    }

    #[test]
    fn test_dei_and_vowel_sequences_after_pre_kar() {
        let mut engine = BornoEngine::new();
        engine.set_mode(InputMode::BornoUnicode);

        // 1. Typing 'দেই': 'c' (ে) + 'l' (দ) + 'g' (্) + 'd' (ি) -> 'দেই'
        assert_eq!(
            engine.handle_key('c', false, false, false),
            EngineAction::Replace {
                backspaces: 0,
                text: "".to_string(),
            }
        );
        assert_eq!(
            engine.handle_key('l', false, false, false),
            EngineAction::Emit("দে".to_string())
        );
        assert_eq!(
            engine.handle_key('g', false, false, false),
            EngineAction::Replace {
                backspaces: 1,
                text: "্".to_string(),
            }
        );
        assert_eq!(
            engine.handle_key('d', false, false, false),
            EngineAction::Replace {
                backspaces: 1,
                text: "েই".to_string(),
            }
        );
        assert_eq!(engine.get_buffer(), "দেই");

        // 2. Typing 'সেই': 'c' (ে) + 'n' (স) + 'g' (্) + 'd' (ি) -> 'সেই'
        engine.handle_key(' ', false, false, false);
        engine.handle_key('c', false, false, false);
        engine.handle_key('n', false, false, false);
        engine.handle_key('g', false, false, false);
        assert_eq!(
            engine.handle_key('d', false, false, false),
            EngineAction::Replace {
                backspaces: 1,
                text: "েই".to_string(),
            }
        );
        assert!(engine.get_buffer().ends_with("সেই"));

        // 3. Typing 'দিই': 'd' (ি) + 'l' (দ) + 'g' (্) + 'd' (ি) -> 'দিই'
        engine.handle_key(' ', false, false, false);
        engine.handle_key('d', false, false, false);
        engine.handle_key('l', false, false, false);
        engine.handle_key('g', false, false, false);
        assert_eq!(
            engine.handle_key('d', false, false, false),
            EngineAction::Replace {
                backspaces: 1,
                text: "িই".to_string(),
            }
        );
        assert!(engine.get_buffer().ends_with("দিই"));

        // 4. Typing 'কেউ': 'c' (ে) + 'j' (ক) + 'g' (্) + 's' (ু) -> 'কেউ'
        engine.handle_key(' ', false, false, false);
        engine.handle_key('c', false, false, false);
        engine.handle_key('j', false, false, false);
        engine.handle_key('g', false, false, false);
        assert_eq!(
            engine.handle_key('s', false, false, false),
            EngineAction::Replace {
                backspaces: 1,
                text: "েউ".to_string(),
            }
        );
        assert!(engine.get_buffer().ends_with("কেউ"));

        // 5. Typing 'দই' (actual 'দ' + 'ই' without 'ে'): 'l' (দ) + 'g' (্) + 'd' (ি) -> 'দই'
        engine.handle_key(' ', false, false, false);
        engine.handle_key('l', false, false, false);
        engine.handle_key('g', false, false, false);
        assert_eq!(
            engine.handle_key('d', false, false, false),
            EngineAction::Replace {
                backspaces: 1,
                text: "ই".to_string(),
            }
        );
        assert!(engine.get_buffer().ends_with("দই"));

        // 6. Backspace after 'দে' + 'g' -> cleanly restores 'দে'
        engine.handle_key(' ', false, false, false);
        engine.handle_key('c', false, false, false);
        engine.handle_key('l', false, false, false); // Screen: 'দে'
        engine.handle_key('g', false, false, false); // Screen: 'দ্'
        assert_eq!(
            engine.handle_key('\x08', false, false, false),
            EngineAction::Replace {
                backspaces: 1,
                text: "ে".to_string(),
            }
        );
        assert!(engine.get_buffer().ends_with("দে"));

        // 7. Space after 'দে' + 'g' -> cleanly restores 'দে '
        engine.handle_key('g', false, false, false); // Screen: 'দ্'
        assert_eq!(
            engine.handle_key(' ', false, false, false),
            EngineAction::Replace {
                backspaces: 1,
                text: "ে ".to_string(),
            }
        );
        assert!(engine.get_buffer().ends_with("দে "));
    }

    #[test]
    fn test_r_phala_and_reph_complex_vowels() {
        let mut engine = BornoEngine::new();
        engine.set_mode(InputMode::BornoUnicode);

        // 1. 'রাত্রি': 'v' (র) + 'f' (া) + 'd' (ি) + 'k' (ত) + 'z' (্র) -> 'রাত্রি'
        engine.handle_key('v', false, false, false);
        engine.handle_key('f', false, false, false);
        engine.handle_key('d', false, false, false); // pending 'ি'
        let act = engine.handle_key('k', false, false, false);
        assert_eq!(act, EngineAction::Emit("তি".to_string()));
        let act = engine.handle_key('z', false, false, false);
        assert_eq!(act, EngineAction::Replace {
            backspaces: 1,
            text: "্রি".to_string(),
        });
        assert_eq!(engine.get_buffer(), "রাত্রি");

        // 2. 'ওতপ্রোত': 'x' (ও) + 'k' (ত) + 'c' (ে) + 'r' (প) + 'z' (্র) + 'f' (া) + 'k' (ত) -> 'ওতপ্রোত'
        engine.handle_key(' ', false, false, false);
        engine.handle_key('x', false, false, false);
        engine.handle_key('k', false, false, false);
        engine.handle_key('c', false, false, false);
        engine.handle_key('r', false, false, false); // emits 'পে'
        let act = engine.handle_key('z', false, false, false);
        assert_eq!(act, EngineAction::Replace {
            backspaces: 1,
            text: "্রে".to_string(),
        });
        let act = engine.handle_key('f', false, false, false);
        assert_eq!(act, EngineAction::Replace {
            backspaces: 1,
            text: "ো".to_string(),
        });
        engine.handle_key('k', false, false, false);
        assert!(engine.get_buffer().ends_with("ওতপ্রোত"));

        // 3. 'ঊর্মি': 'g' (্) + 'S' (ঊ) + 'd' (ি) + 'A' (র্) + 'm' (ম) -> 'ঊর্মি'
        engine.handle_key(' ', false, false, false);
        engine.handle_key('g', false, false, false);
        engine.handle_key('s', true, false, false); // 'ঊ'
        engine.handle_key('d', false, false, false); // pending 'ি'
        let act = engine.handle_key('a', true, false, false); // 'A' (র্)
        assert_eq!(act, EngineAction::Emit("র্".to_string()));
        let act = engine.handle_key('m', false, false, false); // 'ম'
        assert_eq!(act, EngineAction::Emit("মি".to_string()));
        assert!(engine.get_buffer().ends_with("ঊর্মি"));

        // 4. 'ঊর্ধ্বে': 'g' + 'S' (ঊ) + 'c' (ে) + 'A' (র্) + 'L' (ধ) + 'g' + 'h' (্ব) -> 'ঊর্ধ্বে'
        engine.handle_key(' ', false, false, false);
        engine.handle_key('g', false, false, false);
        engine.handle_key('s', true, false, false); // 'ঊ'
        engine.handle_key('c', false, false, false); // pending 'ে'
        engine.handle_key('a', true, false, false); // 'A' (র্)
        let act = engine.handle_key('l', true, false, false); // 'L' (ধ) -> emits 'ধে'
        assert_eq!(act, EngineAction::Emit("ধে".to_string()));
        engine.handle_key('g', false, false, false); // '্'
        engine.handle_key('h', false, false, false); // 'ব'
        assert!(engine.get_buffer().ends_with("ঊর্ধ্বে"));
    }
}

