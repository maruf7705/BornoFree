//! Windows Global Low-Level Keyboard Hook Engine for BornoFree.
//!
//! Provides system-wide zero-lag input interception and SendInput injection
//! across all Windows applications (Chrome, MS Word, Notepad, VS Code, etc.).

use borno_core::{BornoEngine, EngineAction, InputMode};
use colored::*;
use std::sync::Mutex;
use std::time::SystemTime;

// Win32 Constants
const WH_KEYBOARD_LL: i32 = 13;
const WM_KEYDOWN: usize = 0x0100;
const WM_SYSKEYDOWN: usize = 0x0104;

const VK_SHIFT: i32 = 0x10;
const VK_CONTROL: i32 = 0x11;
const VK_MENU: i32 = 0x12; // Alt
const VK_BACK: u16 = 0x08;

const INPUT_KEYBOARD: u32 = 1;
const KEYEVENTF_KEYUP: u32 = 0x0002;
const KEYEVENTF_UNICODE: u32 = 0x0004;

// Magic tag to distinguish our synthetic SendInput from physical keystrokes
const BORNO_EXTRA_INFO: usize = 0xB080_100;

#[repr(C)]
#[derive(Copy, Clone)]
#[allow(non_snake_case)]
struct KEYBDINPUT {
    wVk: u16,
    wScan: u16,
    dwFlags: u32,
    time: u32,
    dwExtraInfo: usize,
}

#[repr(C)]
#[derive(Copy, Clone)]
union INPUT_UNION {
    ki: KEYBDINPUT,
    padding: [u8; 32],
}

#[repr(C)]
struct INPUT {
    r#type: u32,
    _pad: u32,
    u: INPUT_UNION,
}

#[repr(C)]
#[derive(Copy, Clone)]
#[allow(non_snake_case)]
struct KBDLLHOOKSTRUCT {
    vkCode: u32,
    scanCode: u32,
    flags: u32,
    time: u32,
    dwExtraInfo: usize,
}

extern "system" {
    fn SetWindowsHookExW(
        idHook: i32,
        lpfn: unsafe extern "system" fn(i32, usize, isize) -> isize,
        hmod: isize,
        dwThreadId: u32,
    ) -> isize;
    fn UnhookWindowsHookEx(hhk: isize) -> i32;
    fn CallNextHookEx(hhk: isize, nCode: i32, wParam: usize, lParam: isize) -> isize;
    fn GetAsyncKeyState(vKey: i32) -> i16;
    fn SendInput(cInputs: u32, pInputs: *const INPUT, cbSize: i32) -> u32;
    fn PostMessageW(hWnd: isize, Msg: u32, wParam: usize, lParam: isize) -> i32;
}

static ENGINE: Mutex<Option<BornoEngine>> = Mutex::new(None);
static mut HOOK_HANDLE: isize = 0;

/// Returns the currently active input mode.
pub fn get_current_mode() -> InputMode {
    let engine_guard = ENGINE.lock().unwrap();
    if let Some(ref engine) = *engine_guard {
        engine.mode
    } else {
        InputMode::English
    }
}

/// Sets the active input mode from the GUI.
pub fn set_mode(new_mode: InputMode) {
    let mut engine_guard = ENGINE.lock().unwrap();
    if let Some(ref mut engine) = *engine_guard {
        engine.set_mode(new_mode);
    }
}

/// Installs the global low-level keyboard hook.
pub fn install_keyboard_hook() -> bool {
    {
        let mut engine_guard = ENGINE.lock().unwrap();
        *engine_guard = Some(BornoEngine::new());
    }

    unsafe {
        let hook = SetWindowsHookExW(WH_KEYBOARD_LL, keyboard_hook_proc, 0, 0);
        if hook != 0 {
            HOOK_HANDLE = hook;
            true
        } else {
            false
        }
    }
}

/// Uninstalls the global low-level keyboard hook on application shutdown.
pub fn uninstall_keyboard_hook() {
    unsafe {
        if HOOK_HANDLE != 0 {
            UnhookWindowsHookEx(HOOK_HANDLE);
            HOOK_HANDLE = 0;
        }
    }
}

/// Low-level keyboard hook callback procedure.
unsafe extern "system" fn keyboard_hook_proc(n_code: i32, w_param: usize, l_param: isize) -> isize {
    if n_code >= 0 && (w_param == WM_KEYDOWN || w_param == WM_SYSKEYDOWN) {
        let kbd_ptr = l_param as *const KBDLLHOOKSTRUCT;
        let kbd = *kbd_ptr;

        // Skip our own synthetic keystrokes to prevent infinite loops
        if kbd.dwExtraInfo == BORNO_EXTRA_INFO {
            return CallNextHookEx(0, n_code, w_param, l_param);
        }

        // Check modifier keys state
        let shift = (GetAsyncKeyState(VK_SHIFT) as u16 & 0x8000) != 0;
        let ctrl = (GetAsyncKeyState(VK_CONTROL) as u16 & 0x8000) != 0;
        let alt = (GetAsyncKeyState(VK_MENU) as u16 & 0x8000) != 0;

        let mut engine_guard = ENGINE.lock().unwrap();
        if let Some(ref mut engine) = *engine_guard {
            let action = engine.handle_vk(kbd.vkCode, shift, ctrl, alt);

            match action {
                EngineAction::ModeSwitch(new_mode) => {
                    print_mode_change(new_mode);
                    // Notify the GUI floating bar to repaint with the new mode
                    let gui_hwnd = crate::gui::GUI_HWND.load(std::sync::atomic::Ordering::SeqCst);
                    if gui_hwnd != 0 {
                        PostMessageW(gui_hwnd, crate::gui::WM_APP_MODE_CHANGED, 0, 0);
                    }
                    return 1; // Eat the hotkey press
                }
                EngineAction::Emit(text) => {
                    for c in text.chars() {
                        send_unicode_char(c);
                    }
                    return 1; // Eat physical keystroke
                }
                EngineAction::Replace { backspaces, text } => {
                    send_backspaces(backspaces);
                    for c in text.chars() {
                        send_unicode_char(c);
                    }
                    return 1; // Eat physical keystroke
                }
                EngineAction::Passthrough => {}
            }
        }
    }

    CallNextHookEx(0, n_code, w_param, l_param)
}

/// Injects a Unicode character into the focused window.
unsafe fn send_unicode_char(c: char) {
    let mut u16_buf = [0u16; 2];
    let encoded = c.encode_utf16(&mut u16_buf);

    for &code in &*encoded {
        let inputs = [
            INPUT {
                r#type: INPUT_KEYBOARD,
                _pad: 0,
                u: INPUT_UNION {
                    ki: KEYBDINPUT {
                        wVk: 0,
                        wScan: code,
                        dwFlags: KEYEVENTF_UNICODE,
                        time: 0,
                        dwExtraInfo: BORNO_EXTRA_INFO,
                    },
                },
            },
            INPUT {
                r#type: INPUT_KEYBOARD,
                _pad: 0,
                u: INPUT_UNION {
                    ki: KEYBDINPUT {
                        wVk: 0,
                        wScan: code,
                        dwFlags: KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: BORNO_EXTRA_INFO,
                    },
                },
            },
        ];

        SendInput(2, inputs.as_ptr(), std::mem::size_of::<INPUT>() as i32);
    }
}

/// Injects backspace keystrokes to erase prior characters.
unsafe fn send_backspaces(count: usize) {
    for _ in 0..count {
        let inputs = [
            INPUT {
                r#type: INPUT_KEYBOARD,
                _pad: 0,
                u: INPUT_UNION {
                    ki: KEYBDINPUT {
                        wVk: VK_BACK,
                        wScan: 0,
                        dwFlags: 0,
                        time: 0,
                        dwExtraInfo: BORNO_EXTRA_INFO,
                    },
                },
            },
            INPUT {
                r#type: INPUT_KEYBOARD,
                _pad: 0,
                u: INPUT_UNION {
                    ki: KEYBDINPUT {
                        wVk: VK_BACK,
                        wScan: 0,
                        dwFlags: KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: BORNO_EXTRA_INFO,
                    },
                },
            },
        ];

        SendInput(2, inputs.as_ptr(), std::mem::size_of::<INPUT>() as i32);
    }
}

fn print_mode_change(mode: InputMode) {
    let now = chrono_or_simple_time();
    let badge = mode.badge();
    let name = mode.name();

    let colored_name = match mode {
        InputMode::English => name.cyan(),
        InputMode::BornoUnicode => name.green().bold(),
        InputMode::Phonetic => name.magenta().bold(),
        _ => name.white(),
    };

    println!(
        "  [{}] 🔄 Mode Switched: [{}] {}",
        now.dimmed(),
        badge.bold(),
        colored_name
    );
}

fn chrono_or_simple_time() -> String {
    let d = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = d.as_secs();
    let hour = (secs / 3600) % 24;
    let min = (secs / 60) % 60;
    let sec = secs % 60;
    format!("{:02}:{:02}:{:02}", hour, min, sec)
}
