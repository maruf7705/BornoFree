//! C ABI bridge used by the Linux Fcitx5 input-method addon.
//!
//! The API intentionally owns exactly one [`BornoEngine`] per Fcitx input
//! context. This prevents an unfinished pre-kar or conjunct in one window
//! from changing the text entered in another window.

use std::ffi::{c_char, CString};
use std::ptr;

use borno_core::{BornoEngine, EngineAction, InputMode};

/// Action constants shared with `linux/fcitx5/include/borno_ffi.h`.
pub const BORNO_ACTION_PASSTHROUGH: u32 = 0;
pub const BORNO_ACTION_EMIT: u32 = 1;
pub const BORNO_ACTION_REPLACE: u32 = 2;
pub const BORNO_ACTION_MODE_SWITCH: u32 = 3;

/// An owned response from the typing engine.
///
/// `text` is UTF-8 allocated by Rust. The caller must always release a
/// non-null pointer through [`borno_action_free`].
#[repr(C)]
pub struct BornoAction {
    pub kind: u32,
    pub backspaces: usize,
    pub text: *mut c_char,
}

impl BornoAction {
    const fn passthrough() -> Self {
        Self {
            kind: BORNO_ACTION_PASSTHROUGH,
            backspaces: 0,
            text: ptr::null_mut(),
        }
    }
}

fn owned_text(text: String) -> *mut c_char {
    // The engine only produces Unicode keyboard text, never an embedded NUL.
    CString::new(text)
        .expect("BornoFree engine emitted an embedded NUL")
        .into_raw()
}

fn into_ffi_action(action: EngineAction) -> BornoAction {
    match action {
        EngineAction::Passthrough => BornoAction::passthrough(),
        EngineAction::Emit(text) => BornoAction {
            kind: BORNO_ACTION_EMIT,
            backspaces: 0,
            text: owned_text(text),
        },
        EngineAction::Replace { backspaces, text } => BornoAction {
            kind: BORNO_ACTION_REPLACE,
            backspaces,
            text: owned_text(text),
        },
        EngineAction::ModeSwitch(_) => BornoAction {
            kind: BORNO_ACTION_MODE_SWITCH,
            backspaces: 0,
            text: ptr::null_mut(),
        },
    }
}

/// Creates a Borno Unicode engine. Each Fcitx input context owns one handle.
#[no_mangle]
pub extern "C" fn borno_engine_new() -> *mut BornoEngine {
    let mut engine = BornoEngine::new();
    engine.set_mode(InputMode::BornoUnicode);
    Box::into_raw(Box::new(engine))
}

/// Clears unfinished composition and restores Borno Unicode mode.
#[no_mangle]
pub unsafe extern "C" fn borno_engine_reset(engine: *mut BornoEngine) {
    if let Some(engine) = engine.as_mut() {
        engine.set_mode(InputMode::BornoUnicode);
    }
}

/// Sets the input mode for the engine.
/// 0: English, 1: Borno Unicode, 2: Jatiya, 3: Avro Phonetic.
#[no_mangle]
pub unsafe extern "C" fn borno_engine_set_mode(engine: *mut BornoEngine, mode: u32) {
    if let Some(engine) = engine.as_mut() {
        let input_mode = match mode {
            0 => InputMode::English,
            1 => InputMode::BornoUnicode,
            2 => InputMode::Jatiya,
            3 => InputMode::Phonetic,
            _ => InputMode::BornoUnicode,
        };
        engine.set_mode(input_mode);
    }
}

/// Frees an engine allocated by [`borno_engine_new`].
#[no_mangle]
pub unsafe extern "C" fn borno_engine_free(engine: *mut BornoEngine) {
    if !engine.is_null() {
        drop(Box::from_raw(engine));
    }
}

/// Converts one normalized key event into an output action.
///
/// `key` must be an ASCII printable character, Backspace (`8`), Tab (`9`),
/// or Enter (`13`). Modified key chords pass through: Hyprland owns F12,
/// Ctrl+Alt+V, and Ctrl+Alt+E and switches the Fcitx input method globally.
#[no_mangle]
pub unsafe extern "C" fn borno_engine_handle_key(
    engine: *mut BornoEngine,
    key: u32,
    shift: u8,
    ctrl: u8,
    alt: u8,
) -> BornoAction {
    let Some(engine) = engine.as_mut() else {
        return BornoAction::passthrough();
    };

    if ctrl != 0 || alt != 0 {
        return BornoAction::passthrough();
    }

    let Some(key) = char::from_u32(key) else {
        return BornoAction::passthrough();
    };

    into_ffi_action(engine.handle_key(key, shift != 0, false, false))
}

/// Releases text returned in a [`BornoAction`]. Passing null is valid.
#[no_mangle]
pub unsafe extern "C" fn borno_action_free(text: *mut c_char) {
    if !text.is_null() {
        drop(CString::from_raw(text));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;

    unsafe fn text_and_free(action: BornoAction) -> String {
        let text = if action.text.is_null() {
            String::new()
        } else {
            CStr::from_ptr(action.text).to_str().unwrap().to_owned()
        };
        borno_action_free(action.text);
        text
    }

    #[test]
    fn starts_in_borno_unicode_and_returns_owned_utf8() {
        let engine = borno_engine_new();
        let action = unsafe { borno_engine_handle_key(engine, 'j' as u32, 0, 0, 0) };
        assert_eq!(action.kind, BORNO_ACTION_EMIT);
        assert_eq!(unsafe { text_and_free(action) }, "ক");
        unsafe { borno_engine_free(engine) };
    }

    #[test]
    fn replacement_reports_unicode_backspaces() {
        let engine = borno_engine_new();
        let first = unsafe { borno_engine_handle_key(engine, 'g' as u32, 0, 0, 0) };
        assert_eq!(unsafe { text_and_free(first) }, "্");

        let replacement = unsafe { borno_engine_handle_key(engine, 'f' as u32, 0, 0, 0) };
        assert_eq!(replacement.kind, BORNO_ACTION_REPLACE);
        assert_eq!(replacement.backspaces, 1);
        assert_eq!(unsafe { text_and_free(replacement) }, "আ");
        unsafe { borno_engine_free(engine) };
    }

    #[test]
    fn reset_drops_pending_pre_kar() {
        let engine = borno_engine_new();
        let pending = unsafe { borno_engine_handle_key(engine, 'd' as u32, 0, 0, 0) };
        assert_eq!(pending.kind, BORNO_ACTION_REPLACE);
        assert_eq!(unsafe { text_and_free(pending) }, "");

        unsafe { borno_engine_reset(engine) };
        let key = unsafe { borno_engine_handle_key(engine, 'j' as u32, 0, 0, 0) };
        assert_eq!(unsafe { text_and_free(key) }, "ক");
        unsafe { borno_engine_free(engine) };
    }

    #[test]
    fn modified_and_invalid_events_passthrough() {
        let engine = borno_engine_new();
        let modified = unsafe { borno_engine_handle_key(engine, 'v' as u32, 0, 1, 1) };
        assert_eq!(modified.kind, BORNO_ACTION_PASSTHROUGH);
        let invalid = unsafe { borno_engine_handle_key(engine, 0x11_0000, 0, 0, 0) };
        assert_eq!(invalid.kind, BORNO_ACTION_PASSTHROUGH);
        unsafe { borno_engine_free(engine) };
    }

    #[test]
    fn null_engine_is_a_safe_passthrough() {
        let action = unsafe { borno_engine_handle_key(ptr::null_mut(), 'j' as u32, 0, 0, 0) };
        assert_eq!(action.kind, BORNO_ACTION_PASSTHROUGH);
        assert!(action.text.is_null());
    }
}
