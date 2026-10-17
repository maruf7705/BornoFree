//! Bengali Unicode constants and character classification utilities.

// Vowels (স্বরবর্ণ)
pub const BENGALI_A: char = '\u{0985}';    // অ
pub const BENGALI_AA: char = '\u{0986}';   // আ
pub const BENGALI_I: char = '\u{0987}';    // ই
pub const BENGALI_II: char = '\u{0988}';   // ঈ
pub const BENGALI_U: char = '\u{0989}';    // উ
pub const BENGALI_UU: char = '\u{098A}';   // ঊ
pub const BENGALI_R: char = '\u{098B}';    // ঋ
pub const BENGALI_E: char = '\u{098F}';    // এ
pub const BENGALI_AI: char = '\u{0990}';   // ঐ
pub const BENGALI_O: char = '\u{0993}';    // ও
pub const BENGALI_AU: char = '\u{0994}';   // ঔ

// Vowel Signs / Kars (কার চিহ্ন)
pub const KAR_AA: char = '\u{09BE}';       // া
pub const KAR_I: char = '\u{09BF}';        // ি
pub const KAR_II: char = '\u{09C0}';       // ী
pub const KAR_U: char = '\u{09C1}';        // ু
pub const KAR_UU: char = '\u{09C2}';       // ূ
pub const KAR_R: char = '\u{09C3}';        // ৃ
pub const KAR_E: char = '\u{09C7}';        // ে
pub const KAR_AI: char = '\u{09C8}';       // ৈ
pub const KAR_O: char = '\u{09CB}';        // ো
pub const KAR_AU: char = '\u{09CC}';       // ৌ

// Modifiers & Signs
pub const HASANTA: char = '\u{09CD}';      // ্ (Virama / Halant)
pub const ANUSVARA: char = '\u{0982}';     // ং
pub const VISARGA: char = '\u{0983}';      // ঃ
pub const CHANDRABINDU: char = '\u{0981}'; // ঁ
pub const KHANDA_TA: char = '\u{09CE}';    // ৎ
pub const DARI: char = '\u{09E4}';         // ।
pub const DOUBLE_DARI: char = '\u{09E5}';  // ॥
pub const NUKTA: char = '\u{09BC}';        // ়
pub const ZWNJ: char = '\u{200C}';         // Zero-width non-joiner
pub const ZWJ: char = '\u{200D}';          // Zero-width joiner

/// Check if a character is within the Bengali Unicode block (U+0980 to U+09FF).
pub fn is_bengali_char(c: char) -> bool {
    ('\u{0980}'..='\u{09FF}').contains(&c)
}

/// Check if a character is a Bengali independent vowel.
pub fn is_independent_vowel(c: char) -> bool {
    matches!(
        c,
        BENGALI_A
            | BENGALI_AA
            | BENGALI_I
            | BENGALI_II
            | BENGALI_U
            | BENGALI_UU
            | BENGALI_R
            | BENGALI_E
            | BENGALI_AI
            | BENGALI_O
            | BENGALI_AU
    )
}

/// Check if a character is a Bengali dependent vowel sign (Kar).
pub fn is_kar(c: char) -> bool {
    matches!(
        c,
        KAR_AA
            | KAR_I
            | KAR_II
            | KAR_U
            | KAR_UU
            | KAR_R
            | KAR_E
            | KAR_AI
            | KAR_O
            | KAR_AU
    )
}

/// Check if a Kar is visually rendered on the left of the base consonant in traditional orthography.
/// (Pre-kar: ি, ে, ৈ)
pub fn is_pre_kar(c: char) -> bool {
    matches!(c, KAR_I | KAR_E | KAR_AI)
}

/// Check if a Kar is a two-part split vowel sign in traditional visual orthography (ো, ৌ).
pub fn is_split_vowel_kar(c: char) -> bool {
    matches!(c, KAR_O | KAR_AU)
}

/// Check if a character is a combining sign / mark (Chandrabindu, Anusvara, Visarga, Nukta).
pub fn is_combining_mark(c: char) -> bool {
    matches!(c, CHANDRABINDU | ANUSVARA | VISARGA | NUKTA)
}

/// Bengali Au Length Mark (ৗ)
pub const AU_LENGTH_MARK: char = '\u{09D7}';

/// Compose a base consonant and a Nukta into canonical atomic scalar (e.g. ড + ় -> ড়).
pub fn compose_nukta(base: char, nukta: char) -> Option<char> {
    if nukta == NUKTA {
        match base {
            '\u{09A1}' => Some('\u{09DC}'), // ড + ় = ড়
            '\u{09A2}' => Some('\u{09DD}'), // ঢ + ় = ঢ়
            '\u{09AF}' => Some('\u{09DF}'), // য + ় = য়
            _ => None,
        }
    } else {
        None
    }
}

/// Check if a character is a Bengali consonant.
pub fn is_consonant(c: char) -> bool {
    ('\u{0995}'..='\u{09B9}').contains(&c)
        || matches!(c, '\u{09DC}' | '\u{09DD}' | '\u{09DF}' | KHANDA_TA)
}

/// Check if a character is a Bengali digit (০-৯).
pub fn is_bengali_digit(c: char) -> bool {
    ('\u{09E6}'..='\u{09EF}').contains(&c)
}

/// Convert an ASCII digit ('0'-'9') to Bengali digit ('০'-'৯').
pub fn to_bengali_digit(ascii_digit: char) -> Option<char> {
    if ascii_digit.is_ascii_digit() {
        let offset = ascii_digit as u32 - '0' as u32;
        char::from_u32('\u{09E6}' as u32 + offset)
    } else {
        None
    }
}

