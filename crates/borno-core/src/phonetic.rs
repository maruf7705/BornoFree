//! High-performance Avro Phonetic Transliteration Engine in pure Rust.
//!
//! Implements official Avro Phonetic grammar rules, context matching (prefix/suffix,
//! vowel/consonant/punctuation/exact), and live keyboard transliteration.

use serde::Deserialize;
use std::cmp::Ordering;
use std::sync::OnceLock;

/// Match criterion inside an Avro grammar rule.
#[derive(Debug, Clone, Deserialize)]
pub struct RuleMatch {
    #[serde(rename = "type")]
    pub match_type: String, // "prefix" or "suffix"
    pub scope: String,      // "punctuation", "vowel", "consonant", "number", "exact" (with optional '!' prefix)
    #[serde(default)]
    pub value: String,
}

/// Conditional replacement rule.
#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    pub matches: Vec<RuleMatch>,
    pub replace: String,
}

/// Pattern definition in Avro Phonetic grammar.
#[derive(Debug, Clone, Deserialize)]
pub struct Pattern {
    pub find: String,
    pub replace: String,
    #[serde(default)]
    pub rules: Vec<Rule>,
}

/// Full Avro Phonetic Grammar definition.
#[derive(Debug, Clone, Deserialize)]
pub struct AvroGrammar {
    pub vowel: String,
    pub consonant: String,
    pub number: String,
    pub casesensitive: String,
    pub patterns: Vec<Pattern>,
}

/// High-performance Avro Phonetic transliteration engine.
#[derive(Debug, Clone)]
pub struct PhoneticEngine {
    grammar: &'static AvroGrammar,
    max_pattern_len: usize,
}

static AVRO_GRAMMAR: OnceLock<AvroGrammar> = OnceLock::new();

fn get_grammar() -> &'static AvroGrammar {
    AVRO_GRAMMAR.get_or_init(|| {
        let json_data = include_str!("AvroPhonetic.json");
        serde_json::from_str(json_data).expect("Malformed AvroPhonetic.json embedded in borno-core")
    })
}

impl Default for PhoneticEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl PhoneticEngine {
    pub fn new() -> Self {
        let grammar = get_grammar();
        let max_pattern_len = grammar
            .patterns
            .first()
            .map(|p| p.find.len())
            .unwrap_or(5);
        Self {
            grammar,
            max_pattern_len,
        }
    }

    /// Transliterate an entire string or word into standard Bengali Unicode.
    pub fn transliterate(&self, input: &str) -> String {
        self.convert(input)
    }

    /// Convert input English transliteration to Bengali using Avro grammar.
    pub fn convert(&self, input: &str) -> String {
        let fixed = self.fix_string(input);
        let len = fixed.len();
        let mut output = String::with_capacity(len * 3);

        let mut cur = 0;
        while cur < len {
            let start = cur as i32;
            let mut matched = false;

            for chunk_len in (1..=self.max_pattern_len).rev() {
                let end = start + chunk_len as i32;
                if end <= len as i32 {
                    let chunk = &fixed[start as usize..end as usize];

                    // Binary search patterns
                    let mut left: i32 = 0;
                    let mut right = self.grammar.patterns.len() as i32 - 1;
                    while right >= left {
                        let mid = (right + left) / 2;
                        let pattern = &self.grammar.patterns[mid as usize];
                        let find = pattern.find.as_str();

                        if find == chunk {
                            if !pattern.rules.is_empty() {
                                for rule in &pattern.rules {
                                    let mut replace = true;

                                    for m in &rule.matches {
                                        let value = m.value.as_str();
                                        let match_type = m.match_type.as_str();
                                        let mut scope = m.scope.as_str();
                                        let mut is_negative = false;

                                        if scope.starts_with('!') {
                                            is_negative = true;
                                            scope = &scope[1..];
                                        }

                                        let chk = if match_type == "suffix" {
                                            end
                                        } else {
                                            start - 1
                                        };

                                        match scope {
                                            "punctuation" => {
                                                let is_punct = (chk < 0 && match_type == "prefix")
                                                    || (chk >= len as i32 && match_type == "suffix")
                                                    || (chk >= 0
                                                        && chk < len as i32
                                                        && self.is_punctuation(&fixed[chk as usize..=chk as usize]));
                                                if is_punct == is_negative {
                                                    replace = false;
                                                    break;
                                                }
                                            }
                                            "vowel" => {
                                                let is_v = (chk >= 0 && match_type == "prefix")
                                                    || (chk < len as i32 && match_type == "suffix");
                                                let is_v = is_v
                                                    && chk >= 0
                                                    && chk < len as i32
                                                    && self.is_vowel(&fixed[chk as usize..=chk as usize]);
                                                if is_v == is_negative {
                                                    replace = false;
                                                    break;
                                                }
                                            }
                                            "consonant" => {
                                                let is_c = (chk >= 0 && match_type == "prefix")
                                                    || (chk < len as i32 && match_type == "suffix");
                                                let is_c = is_c
                                                    && chk >= 0
                                                    && chk < len as i32
                                                    && self.is_consonant(&fixed[chk as usize..=chk as usize]);
                                                if is_c == is_negative {
                                                    replace = false;
                                                    break;
                                                }
                                            }
                                            "number" => {
                                                let is_num = (chk >= 0 && match_type == "prefix")
                                                    || (chk < len as i32 && match_type == "suffix");
                                                let is_num = is_num
                                                    && chk >= 0
                                                    && chk < len as i32
                                                    && self.is_number(&fixed[chk as usize..=chk as usize]);
                                                if is_num == is_negative {
                                                    replace = false;
                                                    break;
                                                }
                                            }
                                            "exact" => {
                                                let (s, e) = if match_type == "suffix" {
                                                    (end, end + value.len() as i32)
                                                } else {
                                                    (start - value.len() as i32, start)
                                                };
                                                if !self.is_exact(value, &fixed, s, e, is_negative) {
                                                    replace = false;
                                                    break;
                                                }
                                            }
                                            _ => {}
                                        }
                                    }

                                    if replace {
                                        output.push_str(&rule.replace);
                                        cur = (end - 1) as usize;
                                        matched = true;
                                        break;
                                    }
                                }
                            }

                            if matched {
                                break;
                            }

                            // Default pattern replacement
                            output.push_str(&pattern.replace);
                            cur = (end - 1) as usize;
                            matched = true;
                            break;
                        } else if find.len() > chunk.len()
                            || (find.len() == chunk.len() && find.cmp(chunk) == Ordering::Less)
                        {
                            left = mid + 1;
                        } else {
                            right = mid - 1;
                        }
                    }

                    if matched {
                        break;
                    }
                }
            }

            if !matched {
                output.push_str(&fixed[cur..cur + 1]);
            }
            cur += 1;
        }

        output
    }

    fn fix_string(&self, string: &str) -> String {
        string
            .chars()
            .map(|character| {
                if self.is_case_sensitive(character) {
                    character
                } else {
                    character.to_ascii_lowercase()
                }
            })
            .collect()
    }

    fn is_vowel(&self, string: &str) -> bool {
        self.grammar.vowel.contains(&string.to_ascii_lowercase())
    }

    fn is_consonant(&self, string: &str) -> bool {
        self.grammar.consonant.contains(&string.to_ascii_lowercase())
    }

    fn is_case_sensitive(&self, character: char) -> bool {
        self.grammar
            .casesensitive
            .contains(character.to_ascii_lowercase())
    }

    fn is_number(&self, character: &str) -> bool {
        self.grammar.number.contains(character)
    }

    fn is_exact(&self, needle: &str, haystack: &str, start: i32, end: i32, not: bool) -> bool {
        let is_match = start >= 0
            && end <= haystack.len() as i32
            && &haystack[start as usize..end as usize] == needle;
        is_match != not
    }

    fn is_punctuation(&self, character: &str) -> bool {
        !(self.is_vowel(character) || self.is_consonant(character))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_avro_sentence() {
        let engine = PhoneticEngine::new();
        assert_eq!(engine.transliterate("ami banglay gan gai"), "আমি বাংলায় গান গাই");
        assert_eq!(
            engine.transliterate("amader valObasa hoye gel ghas, kheye gel goru ar diye gelo ba^sh"),
            "আমাদের ভালোবাসা হয়ে গেল ঘাস, খেয়ে গেল গরু আর দিয়ে গেল বাঁশ"
        );
    }

    #[test]
    fn test_avro_basic_patterns() {
        let engine = PhoneticEngine::new();
        assert_eq!(engine.transliterate("kkhudratikkhudr"), "ক্ষুদ্রাতিক্ষুদ্র");
        assert_eq!(engine.transliterate("kkh"), "ক্ষ");
        assert_eq!(engine.transliterate("bdh"), "ব্ধ");
        assert_eq!(engine.transliterate("kw"), "ক্ব");
        assert_eq!(engine.transliterate("rZ"), "র‍্য");
        assert_eq!(engine.transliterate("k^"), "কঁ");
        assert_eq!(engine.transliterate("ki^"), "কিঁ");
        assert_eq!(engine.transliterate(",,"), "্‌");
        assert_eq!(engine.transliterate("$"), "৳");
        assert_eq!(engine.transliterate(":"), "ঃ");
    }
}
