//! High-performance Phonetic Transliteration Engine (Avro-style English to Bengali).

use crate::bengali::*;
use std::collections::HashMap;

/// Rule node for Phonetic Prefix-Trie parser.
#[derive(Debug, Clone, Default)]
pub struct PhoneticTrieNode {
    pub is_leaf: bool,
    /// Bengali output if matched.
    pub output: Option<String>,
    pub children: HashMap<char, PhoneticTrieNode>,
}

impl PhoneticTrieNode {
    pub fn insert(&mut self, key: &str, output: &str) {
        let mut curr = self;
        for ch in key.chars() {
            curr = curr.children.entry(ch).or_default();
        }
        curr.is_leaf = true;
        curr.output = Some(output.to_string());
    }

    pub fn find_longest_prefix(&self, input: &str) -> Option<(usize, String)> {
        let mut curr = self;
        let mut last_match = None;

        for (idx, ch) in input.char_indices() {
            if let Some(next) = curr.children.get(&ch) {
                curr = next;
                if let Some(ref out) = curr.output {
                    last_match = Some((idx + ch.len_utf8(), out.clone()));
                }
            } else {
                break;
            }
        }

        last_match
    }
}

/// Transliterates an English phonetic word or sentence into Bengali Unicode.
pub struct PhoneticEngine {
    trie: PhoneticTrieNode,
}

impl Default for PhoneticEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl PhoneticEngine {
    pub fn new() -> Self {
        let mut trie = PhoneticTrieNode::default();
        Self::populate_rules(&mut trie);
        Self { trie }
    }

    fn populate_rules(trie: &mut PhoneticTrieNode) {
        // Multi-char vowel patterns
        trie.insert("aa", "আ");
        trie.insert("ee", "ঈ");
        trie.insert("oo", "ঊ");
        trie.insert("oi", "ঐ");
        trie.insert("ou", "ঔ");

        // Single vowels (word start)
        trie.insert("a", "আ");
        trie.insert("A", "আ");
        trie.insert("i", "ই");
        trie.insert("I", "ঈ");
        trie.insert("u", "উ");
        trie.insert("U", "ঊ");
        trie.insert("e", "এ");
        trie.insert("E", "এ");
        trie.insert("o", "অ");
        trie.insert("O", "ও");

        // Multi-char consonant patterns
        trie.insert("kkh", "ক্ষ");
        trie.insert("kSh", "ক্ষ");
        trie.insert("kS", "ক্ষ");
        trie.insert("kh", "খ");
        trie.insert("gh", "ঘ");
        trie.insert("Ng", "ঙ");
        trie.insert("ch", "চ");
        trie.insert("Ch", "ছ");
        trie.insert("jh", "ঝ");
        trie.insert("Th", "ঠ");
        trie.insert("Dh", "ঢ");
        trie.insert("th", "থ");
        trie.insert("dh", "ধ");
        trie.insert("ph", "ফ");
        trie.insert("bh", "ভ");
        trie.insert("sh", "শ");
        trie.insert("Sh", "ষ");
        trie.insert("Rh", "ঢ়");
        trie.insert("ng", "ং");

        // Single consonants
        trie.insert("k", "ক");
        trie.insert("K", "খ");
        trie.insert("g", "গ");
        trie.insert("G", "ঘ");
        trie.insert("c", "চ");
        trie.insert("j", "জ");
        trie.insert("J", "ঝ");
        trie.insert("T", "ট");
        trie.insert("D", "ড");
        trie.insert("N", "ণ");
        trie.insert("t", "ত");
        trie.insert("d", "দ");
        trie.insert("n", "ন");
        trie.insert("p", "প");
        trie.insert("P", "প");
        trie.insert("f", "ফ");
        trie.insert("b", "ব");
        trie.insert("B", "ভ");
        trie.insert("v", "ভ");
        trie.insert("m", "ম");
        trie.insert("M", "ম");
        trie.insert("z", "য");
        trie.insert("Z", "য");
        trie.insert("r", "র");
        trie.insert("R", "ড়");
        trie.insert("l", "ল");
        trie.insert("L", "ল");
        trie.insert("S", "ষ");
        trie.insert("s", "স");
        trie.insert("h", "হ");
        trie.insert("H", "হ");
        trie.insert("y", "য়");
        trie.insert("Y", "য়");
        trie.insert("w", "ও");
    }

    /// Transliterate an English token into Bengali.
    pub fn transliterate(&self, input: &str) -> String {
        let mut result = String::with_capacity(input.len() * 2);
        let mut remainder = input;
        let mut prev_was_consonant = false;

        while !remainder.is_empty() {
            if let Some((consumed, ref bangla)) = self.trie.find_longest_prefix(remainder) {
                // If previous char was consonant and this is a vowel, convert to Kar
                let first_char = bangla.chars().next().unwrap_or(' ');
                if prev_was_consonant && is_independent_vowel(first_char) {
                    let kar = match first_char {
                        BENGALI_AA => KAR_AA,
                        BENGALI_I => KAR_I,
                        BENGALI_II => KAR_II,
                        BENGALI_U => KAR_U,
                        BENGALI_UU => KAR_UU,
                        BENGALI_R => KAR_R,
                        BENGALI_E => KAR_E,
                        BENGALI_AI => KAR_AI,
                        BENGALI_O => KAR_O,
                        BENGALI_AU => KAR_AU,
                        _ => ' ',
                    };
                    if kar != ' ' {
                        result.push(kar);
                        prev_was_consonant = false;
                        remainder = &remainder[consumed..];
                        continue;
                    }
                }

                result.push_str(bangla);
                prev_was_consonant = bangla.chars().last().map(is_consonant).unwrap_or(false);
                remainder = &remainder[consumed..];
            } else {
                // Passthrough unmatched character (e.g. punctuation, spaces, numbers)
                let c = remainder.chars().next().unwrap();
                result.push(c);
                prev_was_consonant = false;
                remainder = &remainder[c.len_utf8()..];
            }
        }

        result
    }
}
