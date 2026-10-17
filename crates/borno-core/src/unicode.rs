//! Standard Borno Unicode keyboard layout.

use crate::layout::{InputMode, KeyboardLayout};

/// Creates the standard Borno Unicode layout (Ctrl+Alt+V).
pub fn create_borno_unicode_layout() -> KeyboardLayout {
    let mut layout = KeyboardLayout::new("Borno Unicode", InputMode::BornoUnicode)
        .with_hotkey(true, true, false, 'V');

    // Number row
    layout.add_key('1', "১", "!");
    layout.add_key('2', "২", "@");
    layout.add_key('3', "৩", "#");
    layout.add_key('4', "৪", "৳");
    layout.add_key('5', "৫", "%");
    layout.add_key('6', "৬", "^");
    layout.add_key('7', "৭", "ঁ");
    layout.add_key('8', "৮", "*");
    layout.add_key('9', "৯", "(");
    layout.add_key('0', "০", ")");
    layout.add_key('-', "-", "_");
    layout.add_key('=', "=", "+");

    // Top row
    layout.add_key('q', "ঙ", "ং");
    layout.add_key('w', "য", "য়");
    layout.add_key('e', "ড", "ঢ");
    layout.add_key('r', "প", "ফ");
    layout.add_key('t', "ট", "ঠ");
    layout.add_key('y', "চ", "ছ");
    layout.add_key('u', "জ", "ঝ");
    layout.add_key('i', "হ", "ঞ");
    layout.add_key('o', "গ", "ঘ");
    layout.add_key('p', "ড়", "ঢ়");
    layout.add_key('[', "[", "{");
    layout.add_key(']', "]", "}");
    layout.add_key('\\', "ৎ", "ঃ");

    // Home row
    layout.add_key('a', "ৃ", "র্");
    layout.add_key('s', "ু", "ূ");
    layout.add_key('d', "ি", "ী");
    layout.add_key('f', "া", "অ");
    layout.add_key('g', "্", "ৎ");
    layout.add_key('h', "ব", "ভ");
    layout.add_key('j', "ক", "খ");
    layout.add_key('k', "ত", "থ");
    layout.add_key('l', "দ", "ধ");
    layout.add_key(';', ";", ":");
    layout.add_key('\'', "'", "\"");

    // Bottom row
    layout.add_key('z', "্র", "্য");
    layout.add_key('x', "ও", "ৌ");
    layout.add_key('c', "ে", "ৈ");
    layout.add_key('v', "র", "ল");
    layout.add_key('b', "ন", "ণ");
    layout.add_key('n', "স", "ষ");
    layout.add_key('m', "ম", "শ");
    layout.add_key(',', ",", "<");
    layout.add_key('.', "।", ">");
    layout.add_key('/', "/", "?");
    layout.add_key('`', "`", "~");
    layout.add_key(' ', " ", " ");

    // Standard Vowel Composition Rules (Link key 'g' / Hasanta + Kar -> Full Vowel)
    layout.add_conjunct("্া", "আ"); // g + f = আ
    layout.add_conjunct("\u{09CD}\u{09BF}", "ই"); // g + d = ই
    layout.add_conjunct("\u{09CD}\u{09C0}", "ঈ"); // g + D = ঈ
    layout.add_conjunct("্ু", "উ"); // g + s = উ
    layout.add_conjunct("্ূ", "ঊ"); // g + S = ঊ
    layout.add_conjunct("্ৃ", "ঋ"); // g + a = ঋ
    layout.add_conjunct("্ে", "এ"); // g + c = এ
    layout.add_conjunct("\u{09CD}\u{09C8}", "ঐ"); // g + C = ঐ
    layout.add_conjunct("্ও", "ও"); // g + x = ও
    layout.add_conjunct("্ৌ", "ঔ"); // g + X = ঔ
    layout.add_conjunct("্ৗ", "ঔ");
    layout.add_conjunct("ড়্", "ড়");
    layout.add_conjunct("ঢ়্", "ঢ়");
    layout.add_conjunct("অা", "আ");

    // Two-part vowel signs (E-kar + A-kar = O-kar, E-kar + OU-sign = OU-kar)
    layout.add_conjunct("ো", "ো"); // U+09C7 + U+09BE -> U+09CB (O-kar)
    layout.add_conjunct("েৌ", "ৌ"); // U+09C7 + U+09CC -> U+09CC (OU-kar)
    layout.add_conjunct("ৌ", "ৌ"); // U+09C7 + U+09D7 -> U+09CC (OU-kar)
    layout.add_conjunct("েো", "ো");
    layout.add_conjunct("েঁা", "োঁ");
    layout.add_conjunct("েঁৗ", "ৌঁ");

    // Combining Chandrabindu
    layout.add_conjunct("ঁা", "াঁ");
    layout.add_conjunct("ঁু", "ুঁ");
    layout.add_conjunct("ঁূ", "ূঁ");
    layout.add_conjunct("ঁৃ", "ৃঁ");

    // Pre/Post-Vowel inversion fixes
    layout.add_conjunct("ু্র", "্রু");
    layout.add_conjunct("ু্য", "্যু");
    layout.add_conjunct("ূ্র", "্রূ");
    layout.add_conjunct("ূ্য", "্যূ");
    layout.add_conjunct("ৃ্র", "্রৃ");
    layout.add_conjunct("ৃ্য", "্যৃ");
    layout.add_conjunct("্্", "্");

    layout
}
