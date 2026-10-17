//! Native Windows GUI Floating Toolbar and System Tray implementation for BornoFree.
//!
//! Authentic BornoFree Light and Dark UI floating toolbar in pure Rust with NO console window
//! (IMAGE_SUBSYSTEM_WINDOWS_GUI), embedded high-DPI book logo and vector button assets,
//! zero runtime dependencies, and zero input lag.

use borno_core::InputMode;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};

pub const WM_APP_MODE_CHANGED: u32 = 0x8000 + 1;
const WM_APP_TRAY: u32 = 0x8000 + 2;

const WS_POPUP: u32 = 0x80000000;
const WS_VISIBLE: u32 = 0x10000000;
const WS_EX_TOPMOST: u32 = 0x00000008;
const WS_EX_TOOLWINDOW: u32 = 0x00000080;

const WM_CREATE: u32 = 0x0001;
const WM_DESTROY: u32 = 0x0002;
const WM_PAINT: u32 = 0x000F;
const WM_LBUTTONDOWN: u32 = 0x0201;
const WM_LBUTTONDBLCLK: u32 = 0x0203;
const WM_RBUTTONUP: u32 = 0x0205;
const WM_COMMAND: u32 = 0x0111;
const WM_NCLBUTTONDOWN: u32 = 0x00A1;
const HTCAPTION: usize = 2;

const NIM_ADD: u32 = 0x00000000;
const NIM_MODIFY: u32 = 0x00000001;
const NIM_DELETE: u32 = 0x00000002;
const NIF_MESSAGE: u32 = 0x00000001;
const NIF_ICON: u32 = 0x00000002;
const NIF_TIP: u32 = 0x00000004;

const MF_STRING: u32 = 0x00000000;
const MF_CHECKED: u32 = 0x00000008;
const MF_UNCHECKED: u32 = 0x00000000;
const MF_SEPARATOR: u32 = 0x00000800;
const TPM_RIGHTBUTTON: u32 = 0x0002;

const DT_LEFT: u32 = 0x00000000;
const DT_CENTER: u32 = 0x00000001;
const DT_VCENTER: u32 = 0x00000004;
const DT_SINGLELINE: u32 = 0x00000020;
const TRANSPARENT: i32 = 1;

const SW_HIDE: i32 = 0;
const SW_SHOW: i32 = 5;

const DI_NORMAL: u32 = 0x0003;
const SRCCOPY: u32 = 0x00CC0020;
const PS_SOLID: i32 = 0;

const HWND_TOPMOST: isize = -1;
const HWND_NOTOPMOST: isize = -2;
const SWP_NOMOVE: u32 = 0x0002;
const SWP_NOSIZE: u32 = 0x0001;
const MB_OK: u32 = 0x00000000;
const MB_ICONINFORMATION: u32 = 0x00000040;

// Menu IDs
const ID_MENU_ENGLISH: usize = 1001;
const ID_MENU_UNICODE: usize = 1003;
const ID_MENU_TOGGLE_BAR: usize = 1004;
const ID_MENU_TOGGLE_THEME: usize = 1005;
const ID_MENU_TOGGLE_TOPMOST: usize = 1006;
const ID_MENU_ABOUT: usize = 1007;
const ID_MENU_EXIT: usize = 1008;
const ID_MENU_TOGGLE_AUTOSTART: usize = 1009;

// Global GUI State
pub static GUI_HWND: AtomicIsize = AtomicIsize::new(0);
static IS_BAR_VISIBLE: AtomicBool = AtomicBool::new(true);
pub static IS_DARK_THEME: AtomicBool = AtomicBool::new(true); // Default: Sleek Pro Dark Toolbar
static IS_TOPMOST: AtomicBool = AtomicBool::new(true);

// Embedded Authentic BornoFree Assets
const BORNOFREE_BOOK_PNG: &[u8] = include_bytes!("../icons/bornofree_book.png");
const BRAND_DARK_PNG: &[u8] = include_bytes!("../icons/bornofree_brand_dark.png");
const BRAND_LIGHT_PNG: &[u8] = include_bytes!("../icons/bornofree_brand_light.png");
const BTN_CHECK_DARK_PNG: &[u8] = include_bytes!("../icons/btn_check_dark.png");
const BTN_CHECK_LIGHT_PNG: &[u8] = include_bytes!("../icons/btn_check_light.png");
const BTN_GEAR_DARK_PNG: &[u8] = include_bytes!("../icons/btn_gear_dark.png");
const BTN_GEAR_LIGHT_PNG: &[u8] = include_bytes!("../icons/btn_gear_light.png");
const BTN_CLOSE_DARK_PNG: &[u8] = include_bytes!("../icons/btn_close_dark.png");
const BTN_CLOSE_LIGHT_PNG: &[u8] = include_bytes!("../icons/btn_close_light.png");

// Cached HICON handles
static HICON_TRAY: AtomicIsize = AtomicIsize::new(0);
static HICON_BRAND_DARK: AtomicIsize = AtomicIsize::new(0);
static HICON_BRAND_LIGHT: AtomicIsize = AtomicIsize::new(0);
static HICON_CHECK_DARK: AtomicIsize = AtomicIsize::new(0);
static HICON_CHECK_LIGHT: AtomicIsize = AtomicIsize::new(0);
static HICON_GEAR_DARK: AtomicIsize = AtomicIsize::new(0);
static HICON_GEAR_LIGHT: AtomicIsize = AtomicIsize::new(0);
static HICON_CLOSE_DARK: AtomicIsize = AtomicIsize::new(0);
static HICON_CLOSE_LIGHT: AtomicIsize = AtomicIsize::new(0);

#[repr(C)]
#[allow(non_snake_case)]
struct WNDCLASSEXW {
    cbSize: u32,
    style: u32,
    lpfnWndProc: unsafe extern "system" fn(isize, u32, usize, isize) -> isize,
    cbClsExtra: i32,
    cbWndExtra: i32,
    hInstance: isize,
    hIcon: isize,
    hCursor: isize,
    hbrBackground: isize,
    lpszMenuName: *const u16,
    lpszClassName: *const u16,
    hIconSm: isize,
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
struct POINT {
    x: i32,
    y: i32,
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
struct RECT {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[repr(C)]
#[allow(non_snake_case)]
struct PAINTSTRUCT {
    hdc: isize,
    fErase: i32,
    rcPaint: RECT,
    fRestore: i32,
    fIncUpdate: i32,
    rgbReserved: [u8; 32],
}

#[repr(C)]
#[allow(non_snake_case)]
struct NOTIFYICONDATAW {
    cbSize: u32,
    hWnd: isize,
    uID: u32,
    uFlags: u32,
    uCallbackMessage: u32,
    hIcon: isize,
    szTip: [u16; 128],
    dwState: u32,
    dwStateMask: u32,
    szInfo: [u16; 256],
    uTimeoutOrVersion: u32,
    szInfoTitle: [u16; 64],
    dwInfoFlags: u32,
}

#[link(name = "user32")]
#[link(name = "gdi32")]
#[link(name = "shell32")]
extern "system" {
    fn RegisterClassExW(lpwcx: *const WNDCLASSEXW) -> u16;
    fn CreateWindowExW(
        dwExStyle: u32,
        lpClassName: *const u16,
        lpWindowName: *const u16,
        dwStyle: u32,
        X: i32,
        Y: i32,
        nWidth: i32,
        nHeight: i32,
        hWndParent: isize,
        hMenu: isize,
        hInstance: isize,
        lpParam: *mut std::ffi::c_void,
    ) -> isize;
    fn DestroyWindow(hWnd: isize) -> i32;
    fn DefWindowProcW(hWnd: isize, Msg: u32, wParam: usize, lParam: isize) -> isize;
    fn ShowWindow(hWnd: isize, nCmdShow: i32) -> i32;
    fn UpdateWindow(hWnd: isize) -> i32;
    fn InvalidateRect(hWnd: isize, lpRect: *const RECT, bErase: i32) -> i32;
    fn GetClientRect(hWnd: isize, lpRect: *mut RECT) -> i32;
    fn GetSystemMetrics(nIndex: i32) -> i32;
    fn ReleaseCapture() -> i32;
    fn SendMessageW(hWnd: isize, Msg: u32, wParam: usize, lParam: isize) -> isize;
    fn BeginPaint(hWnd: isize, lpPaint: *mut PAINTSTRUCT) -> isize;
    fn EndPaint(hWnd: isize, lpPaint: *const PAINTSTRUCT) -> i32;
    fn FillRect(hDC: isize, lprc: *const RECT, hbr: isize) -> i32;
    fn DrawTextW(hDC: isize, lpchText: *const u16, cchText: i32, lprc: *mut RECT, format: u32) -> i32;
    fn CreateSolidBrush(color: u32) -> isize;
    fn DeleteObject(ho: isize) -> i32;
    fn SelectObject(hdc: isize, h: isize) -> isize;
    fn SetTextColor(hdc: isize, color: u32) -> u32;
    fn SetBkMode(hdc: isize, mode: i32) -> i32;
    fn CreateFontW(
        cHeight: i32,
        cWidth: i32,
        cEscapement: i32,
        cOrientation: i32,
        cWeight: i32,
        bItalic: u32,
        bUnderline: u32,
        bStrikeOut: u32,
        iCharSet: u32,
        iOutPrecision: u32,
        iClipPrecision: u32,
        iQuality: u32,
        iPitchAndFamily: u32,
        pszFaceName: *const u16,
    ) -> isize;
    fn CreatePopupMenu() -> isize;
    fn AppendMenuW(hMenu: isize, uFlags: u32, uIDNewItem: usize, lpNewItem: *const u16) -> i32;
    fn TrackPopupMenuEx(hMenu: isize, uFlags: u32, x: i32, y: i32, hwnd: isize, lptpm: *mut std::ffi::c_void) -> i32;
    fn DestroyMenu(hMenu: isize) -> i32;
    fn GetCursorPos(lpPoint: *mut POINT) -> i32;
    fn ClientToScreen(hWnd: isize, lpPoint: *mut POINT) -> i32;
    fn SetForegroundWindow(hWnd: isize) -> i32;
    fn PostQuitMessage(nExitCode: i32);
    fn Shell_NotifyIconW(dwMessage: u32, lpData: *const NOTIFYICONDATAW) -> i32;
    fn LoadCursorW(hInstance: isize, lpCursorName: *const u16) -> isize;
    fn CreateIconFromResourceEx(
        pbIconBits: *const u8,
        cbIconBits: u32,
        fIcon: i32,
        dwVersion: u32,
        cxDesired: i32,
        cyDesired: i32,
        uFlags: u32,
    ) -> isize;
    fn DrawIconEx(
        hdc: isize,
        xLeft: i32,
        yTop: i32,
        hIcon: isize,
        cxWidth: i32,
        cyWidth: i32,
        istepIfAniCur: u32,
        hbrFlickerFreeDraw: isize,
        diFlags: u32,
    ) -> i32;
    fn CreateCompatibleDC(hdc: isize) -> isize;
    fn CreateCompatibleBitmap(hdc: isize, cx: i32, cy: i32) -> isize;
    fn BitBlt(
        hdcDest: isize,
        nXDest: i32,
        nYDest: i32,
        nWidth: i32,
        nHeight: i32,
        hdcSrc: isize,
        nXSrc: i32,
        nYSrc: i32,
        dwRop: u32,
    ) -> i32;
    fn CreateRoundRectRgn(x1: i32, y1: i32, x2: i32, y2: i32, w: i32, h: i32) -> isize;
    fn SetWindowRgn(hWnd: isize, hRgn: isize, bRedraw: i32) -> i32;
    fn RoundRect(hdc: isize, left: i32, top: i32, right: i32, bottom: i32, width: i32, height: i32) -> i32;
    fn CreatePen(iStyle: i32, cWidth: i32, color: u32) -> isize;
    fn MoveToEx(hdc: isize, x: i32, y: i32, lppt: *mut std::ffi::c_void) -> i32;
    fn LineTo(hdc: isize, x: i32, y: i32) -> i32;
    fn SetWindowPos(hWnd: isize, hWndInsertAfter: isize, X: i32, Y: i32, cx: i32, cy: i32, uFlags: u32) -> i32;
    fn MessageBoxW(hWnd: isize, lpText: *const u16, lpCaption: *const u16, uType: u32) -> i32;
}

fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

// Convert RGB to Win32 COLORREF (0x00BBGGRR)
fn rgb(r: u8, g: u8, b: u8) -> u32 {
    (r as u32) | ((g as u32) << 8) | ((b as u32) << 16)
}

/// Creates an HICON directly from raw PNG bytes using CreateIconFromResourceEx.
pub fn load_png_as_icon(bytes: &[u8], w: i32, h: i32) -> isize {
    unsafe {
        CreateIconFromResourceEx(
            bytes.as_ptr(),
            bytes.len() as u32,
            1, // TRUE for icon
            0x00030000,
            w,
            h,
            0,
        )
    }
}

pub fn get_or_load_tray_icon() -> isize {
    let mut h = HICON_TRAY.load(Ordering::SeqCst);
    if h == 0 {
        h = load_png_as_icon(BORNOFREE_BOOK_PNG, 32, 32);
        HICON_TRAY.store(h, Ordering::SeqCst);
    }
    h
}

pub fn get_or_load_brand_icon(dark: bool) -> isize {
    if dark {
        let mut h = HICON_BRAND_DARK.load(Ordering::SeqCst);
        if h == 0 {
            h = load_png_as_icon(BRAND_DARK_PNG, 130, 36);
            HICON_BRAND_DARK.store(h, Ordering::SeqCst);
        }
        h
    } else {
        let mut h = HICON_BRAND_LIGHT.load(Ordering::SeqCst);
        if h == 0 {
            h = load_png_as_icon(BRAND_LIGHT_PNG, 130, 36);
            HICON_BRAND_LIGHT.store(h, Ordering::SeqCst);
        }
        h
    }
}

pub fn get_or_load_check_icon(dark: bool) -> isize {
    if dark {
        let mut h = HICON_CHECK_DARK.load(Ordering::SeqCst);
        if h == 0 {
            h = load_png_as_icon(BTN_CHECK_DARK_PNG, 28, 28);
            HICON_CHECK_DARK.store(h, Ordering::SeqCst);
        }
        h
    } else {
        let mut h = HICON_CHECK_LIGHT.load(Ordering::SeqCst);
        if h == 0 {
            h = load_png_as_icon(BTN_CHECK_LIGHT_PNG, 28, 28);
            HICON_CHECK_LIGHT.store(h, Ordering::SeqCst);
        }
        h
    }
}

pub fn get_or_load_gear_icon(dark: bool) -> isize {
    if dark {
        let mut h = HICON_GEAR_DARK.load(Ordering::SeqCst);
        if h == 0 {
            h = load_png_as_icon(BTN_GEAR_DARK_PNG, 24, 24);
            HICON_GEAR_DARK.store(h, Ordering::SeqCst);
        }
        h
    } else {
        let mut h = HICON_GEAR_LIGHT.load(Ordering::SeqCst);
        if h == 0 {
            h = load_png_as_icon(BTN_GEAR_LIGHT_PNG, 24, 24);
            HICON_GEAR_LIGHT.store(h, Ordering::SeqCst);
        }
        h
    }
}

pub fn get_or_load_close_icon(dark: bool) -> isize {
    if dark {
        let mut h = HICON_CLOSE_DARK.load(Ordering::SeqCst);
        if h == 0 {
            h = load_png_as_icon(BTN_CLOSE_DARK_PNG, 24, 24);
            HICON_CLOSE_DARK.store(h, Ordering::SeqCst);
        }
        h
    } else {
        let mut h = HICON_CLOSE_LIGHT.load(Ordering::SeqCst);
        if h == 0 {
            h = load_png_as_icon(BTN_CLOSE_LIGHT_PNG, 24, 24);
            HICON_CLOSE_LIGHT.store(h, Ordering::SeqCst);
        }
        h
    }
}

/// Window procedure handling events for the BornoFree Floating Bar.
unsafe extern "system" fn bar_wnd_proc(hwnd: isize, msg: u32, w_param: usize, l_param: isize) -> isize {
    match msg {
        WM_CREATE => {
            setup_tray_icon(hwnd);
            0
        }
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = std::mem::zeroed();
            let screen_dc = BeginPaint(hwnd, &mut ps);

            let mut rect: RECT = std::mem::zeroed();
            GetClientRect(hwnd, &mut rect);
            let width = rect.right - rect.left;
            let height = rect.bottom - rect.top;

            let mem_dc = CreateCompatibleDC(screen_dc);
            let mem_bmp = CreateCompatibleBitmap(screen_dc, width, height);
            let old_bmp = SelectObject(mem_dc, mem_bmp);

            let is_dark = IS_DARK_THEME.load(Ordering::SeqCst);

            // Palette definitions matching BornoFree Light and Dark Toolbars
            let (bg_color, border_color, div_color, com_bg, com_border, text_color, chevron_color) = if is_dark {
                (
                    rgb(22, 27, 34),   // Dark background #161B22
                    rgb(46, 56, 70),   // Dark border #2E3846
                    rgb(38, 48, 62),   // Divider #26303E
                    rgb(30, 37, 48),   // ComboBox bg #1E2530
                    rgb(44, 55, 71),   // ComboBox border #2C3747
                    rgb(245, 247, 250),// Text white
                    rgb(180, 190, 205),// Chevron light slate
                )
            } else {
                (
                    rgb(255, 255, 255), // Light background #FFFFFF
                    rgb(226, 228, 233), // Light border #E2E4E9
                    rgb(229, 231, 235), // Divider #E5E7EB
                    rgb(255, 255, 255), // ComboBox bg #FFFFFF
                    rgb(209, 213, 219), // ComboBox border #D1D5DB
                    rgb(17, 24, 39),    // Text dark navy #111827
                    rgb(75, 85, 99),    // Chevron dark grey #4B5563
                )
            };

            // 1. Fill Outer Capsule Background
            let bg_brush = CreateSolidBrush(bg_color);
            FillRect(mem_dc, &rect, bg_brush);
            DeleteObject(bg_brush);

            // Draw 1px Capsule Outer Border
            let border_pen = CreatePen(PS_SOLID, 1, border_color);
            let old_pen = SelectObject(mem_dc, border_pen);
            let border_brush = CreateSolidBrush(bg_color);
            let old_brush = SelectObject(mem_dc, border_brush);
            RoundRect(mem_dc, 0, 0, width, height, 18, 18);
            SelectObject(mem_dc, old_brush);
            DeleteObject(border_brush);
            SelectObject(mem_dc, old_pen);
            DeleteObject(border_pen);

            // 2. Draw BornoFree Brand Banner (Logo + "BornoFree" text)
            let h_brand = get_or_load_brand_icon(is_dark);
            if h_brand != 0 {
                DrawIconEx(mem_dc, 8, 4, h_brand, 130, 34, 0, 0, DI_NORMAL);
            }

            // Draw Divider 1 (X = 144, Y = 10..32)
            let div_pen = CreatePen(PS_SOLID, 1, div_color);
            let old_div_pen = SelectObject(mem_dc, div_pen);
            MoveToEx(mem_dc, 144, 10, std::ptr::null_mut());
            LineTo(mem_dc, 144, 32);

            // 3. Draw ComboBox Mode Selector Box (X = 152..282, Y = 7..35, 130x28px)
            let com_border_pen = CreatePen(PS_SOLID, 1, com_border);
            let old_com_pen = SelectObject(mem_dc, com_border_pen);
            let com_brush = CreateSolidBrush(com_bg);
            let old_com_brush = SelectObject(mem_dc, com_brush);
            RoundRect(mem_dc, 152, 7, 282, 35, 8, 8);
            SelectObject(mem_dc, old_com_brush);
            DeleteObject(com_brush);
            SelectObject(mem_dc, old_com_pen);
            DeleteObject(com_border_pen);

            // Inside ComboBox: Draw `[ A ]` or `[ অ ]` Badge Outline
            let current_mode = crate::windows_hook::get_current_mode();
            let badge_pen = CreatePen(PS_SOLID, 1, if is_dark { rgb(56, 70, 90) } else { rgb(180, 185, 195) });
            let old_badge_pen = SelectObject(mem_dc, badge_pen);
            RoundRect(mem_dc, 158, 11, 176, 31, 4, 4);
            SelectObject(mem_dc, old_badge_pen);
            DeleteObject(badge_pen);

            // Badge letter ("A" in English mode, "অ" in Bengali modes)
            let font_name = to_wide("Segoe UI");
            let badge_font = CreateFontW(13, 0, 0, 0, 700, 0, 0, 0, 1, 0, 0, 5, 0, font_name.as_ptr());
            let old_font = SelectObject(mem_dc, badge_font);
            SetBkMode(mem_dc, TRANSPARENT);
            SetTextColor(mem_dc, text_color);

            let badge_text = if current_mode == InputMode::English {
                to_wide("A")
            } else {
                to_wide("অ")
            };
            let mut badge_rc = RECT { left: 158, top: 11, right: 176, bottom: 31 };
            DrawTextW(mem_dc, badge_text.as_ptr(), -1, &mut badge_rc, DT_CENTER | DT_VCENTER | DT_SINGLELINE);

            // ComboBox Mode Text
            let text_font = CreateFontW(14, 0, 0, 0, 500, 0, 0, 0, 1, 0, 0, 5, 0, font_name.as_ptr());
            SelectObject(mem_dc, text_font);
            let mode_str = match current_mode {
                InputMode::English => "English",
                InputMode::BornoUnicode => "বর্ণ ইউনিকোড",
                _ => "English",
            };
            let mut text_rect = RECT {
                left: 182,
                top: 7,
                right: 265,
                bottom: 35,
            };
            let mode_text = to_wide(mode_str);
            DrawTextW(mem_dc, mode_text.as_ptr(), -1, &mut text_rect, DT_LEFT | DT_VCENTER | DT_SINGLELINE);

            // Downward Chevron `▾` on right of ComboBox
            let mut arrow_rect = RECT {
                left: 265,
                top: 8,
                right: 279,
                bottom: 34,
            };
            SetTextColor(mem_dc, chevron_color);
            let arrow_text = to_wide("▾");
            DrawTextW(mem_dc, arrow_text.as_ptr(), -1, &mut arrow_rect, DT_CENTER | DT_VCENTER | DT_SINGLELINE);

            SelectObject(mem_dc, old_font);
            DeleteObject(badge_font);
            DeleteObject(text_font);

            // 4. Draw Green Check Action Button (X = 290, Y = 7, 28x28)
            let h_check = get_or_load_check_icon(is_dark);
            if h_check != 0 {
                DrawIconEx(mem_dc, 290, 7, h_check, 28, 28, 0, 0, DI_NORMAL);
            }

            // Draw Divider 2 (X = 328, Y = 10..32)
            SelectObject(mem_dc, div_pen);
            MoveToEx(mem_dc, 328, 10, std::ptr::null_mut());
            LineTo(mem_dc, 328, 32);

            // 5. Draw Settings Gear Button (X = 338, Y = 8, 26x26)
            let h_gear = get_or_load_gear_icon(is_dark);
            if h_gear != 0 {
                DrawIconEx(mem_dc, 338, 8, h_gear, 26, 26, 0, 0, DI_NORMAL);
            }

            // Draw Divider 3 (X = 374, Y = 10..32)
            MoveToEx(mem_dc, 374, 10, std::ptr::null_mut());
            LineTo(mem_dc, 374, 32);

            SelectObject(mem_dc, old_div_pen);
            DeleteObject(div_pen);

            // 6. Draw Close Button (X = 384, Y = 8, 26x26)
            let h_close = get_or_load_close_icon(is_dark);
            if h_close != 0 {
                DrawIconEx(mem_dc, 384, 8, h_close, 26, 26, 0, 0, DI_NORMAL);
            }

            // BitBlt final composited image to screen (100% flicker-free)
            BitBlt(screen_dc, 0, 0, width, height, mem_dc, 0, 0, SRCCOPY);

            SelectObject(mem_dc, old_bmp);
            DeleteObject(mem_bmp);
            DeleteObject(mem_dc);

            EndPaint(hwnd, &ps);
            0
        }
        WM_LBUTTONDOWN => {
            let x = (l_param & 0xFFFF) as i32;

            // 1. Click on Close Button `✕` (380px..416px) -> Minimize to System Tray!
            if x >= 380 {
                ShowWindow(hwnd, SW_HIDE);
                IS_BAR_VISIBLE.store(false, Ordering::SeqCst);
                return 0;
            }

            // 2. Click on Settings Gear `⚙` (332px..372px) -> Pop Settings Menu!
            if x >= 332 && x < 372 {
                let mut pt = POINT { x: 345, y: 44 };
                ClientToScreen(hwnd, &mut pt);
                show_settings_popup_menu(hwnd, pt.x, pt.y);
                return 0;
            }

            // 3. Click on Green Check Button `✓` (286px..324px) -> Quick toggle mode!
            if x >= 286 && x < 324 {
                let cur = crate::windows_hook::get_current_mode();
                let next_mode = if cur == InputMode::English {
                    InputMode::BornoUnicode
                } else {
                    InputMode::English
                };
                crate::windows_hook::set_mode(next_mode);
                InvalidateRect(hwnd, std::ptr::null(), 1);
                update_tray_icon_and_tooltip(hwnd);
                return 0;
            }

            // 4. Click on ComboBox (150px..284px) -> Pop Layout Selection Menu!
            if x >= 150 && x < 284 {
                let mut pt = POINT { x: 152, y: 44 };
                ClientToScreen(hwnd, &mut pt);
                show_mode_popup_menu(hwnd, pt.x, pt.y);
                return 0;
            }

            // 5. Click on Brand / Background (< 150px) -> DRAG WINDOW!
            ReleaseCapture();
            SendMessageW(hwnd, WM_NCLBUTTONDOWN, HTCAPTION, 0);
            0
        }
        WM_RBUTTONUP => {
            // Right-click anywhere on the floating toolbar -> Pop settings menu
            let mut pt: POINT = std::mem::zeroed();
            GetCursorPos(&mut pt);
            SetForegroundWindow(hwnd);
            show_tray_context_menu(hwnd, pt.x, pt.y);
            0
        }
        WM_COMMAND => {
            let id = w_param & 0xFFFF;
            match id {
                ID_MENU_ENGLISH => {
                    crate::windows_hook::set_mode(InputMode::English);
                    InvalidateRect(hwnd, std::ptr::null(), 1);
                    update_tray_icon_and_tooltip(hwnd);
                }
                ID_MENU_UNICODE => {
                    crate::windows_hook::set_mode(InputMode::BornoUnicode);
                    InvalidateRect(hwnd, std::ptr::null(), 1);
                    update_tray_icon_and_tooltip(hwnd);
                }
                ID_MENU_TOGGLE_BAR => {
                    let visible = IS_BAR_VISIBLE.load(Ordering::SeqCst);
                    if visible {
                        ShowWindow(hwnd, SW_HIDE);
                        IS_BAR_VISIBLE.store(false, Ordering::SeqCst);
                    } else {
                        ShowWindow(hwnd, SW_SHOW);
                        SetForegroundWindow(hwnd);
                        IS_BAR_VISIBLE.store(true, Ordering::SeqCst);
                    }
                }
                ID_MENU_TOGGLE_THEME => {
                    let is_dark = !IS_DARK_THEME.load(Ordering::SeqCst);
                    IS_DARK_THEME.store(is_dark, Ordering::SeqCst);
                    InvalidateRect(hwnd, std::ptr::null(), 1);
                    UpdateWindow(hwnd);
                }
                ID_MENU_TOGGLE_TOPMOST => {
                    let topmost = !IS_TOPMOST.load(Ordering::SeqCst);
                    IS_TOPMOST.store(topmost, Ordering::SeqCst);
                    let insert_after = if topmost { HWND_TOPMOST } else { HWND_NOTOPMOST };
                    SetWindowPos(hwnd, insert_after, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
                }
                ID_MENU_ABOUT => {
                    let title = to_wide("About BornoFree (বর্ণফ্রি)");
                    let msg = to_wide(
                        "BornoFree v1.0.0\n\
                        Modern Bengali Typing Engine for Windows\n\n\
                        Keyboard Shortcuts:\n\
                        • Ctrl + Alt + V : বর্ণ ইউনিকোড (Borno Unicode)\n\
                        • Ctrl + Alt + E : English\n\n\
                        Crafted with Rust for 0-lag and high performance."
                    );
                    MessageBoxW(hwnd, msg.as_ptr(), title.as_ptr(), MB_OK | MB_ICONINFORMATION);
                }
                ID_MENU_TOGGLE_AUTOSTART => {
                    let _ = toggle_autostart();
                }
                ID_MENU_EXIT => {
                    remove_tray_icon(hwnd);
                    DestroyWindow(hwnd);
                    PostQuitMessage(0);
                }
                _ => {}
            }
            0
        }
        WM_APP_MODE_CHANGED => {
            // Mode changed via global hotkey (Ctrl+Alt+V / B / E)
            InvalidateRect(hwnd, std::ptr::null(), 1);
            update_tray_icon_and_tooltip(hwnd);
            0
        }
        WM_APP_TRAY => {
            let event = (l_param & 0xFFFF) as u32;
            if event == WM_RBUTTONUP {
                // Right click on System Tray icon -> Show context menu
                let mut pt: POINT = std::mem::zeroed();
                GetCursorPos(&mut pt);
                SetForegroundWindow(hwnd);
                show_tray_context_menu(hwnd, pt.x, pt.y);
            } else if event == WM_LBUTTONDOWN || event == WM_LBUTTONDBLCLK {
                // Left click or Double click on System Tray icon -> Restore / Toggle floating bar!
                let visible = IS_BAR_VISIBLE.load(Ordering::SeqCst);
                if visible {
                    ShowWindow(hwnd, SW_HIDE);
                    IS_BAR_VISIBLE.store(false, Ordering::SeqCst);
                } else {
                    ShowWindow(hwnd, SW_SHOW);
                    SetForegroundWindow(hwnd);
                    IS_BAR_VISIBLE.store(true, Ordering::SeqCst);
                }
            }
            0
        }
        WM_DESTROY => {
            remove_tray_icon(hwnd);
            crate::windows_hook::uninstall_keyboard_hook();
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, w_param, l_param),
    }
}

pub fn is_autostart_enabled() -> bool {
    std::process::Command::new("reg")
        .args(["query", "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run", "/v", "BornoFree"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn toggle_autostart() -> bool {
    let currently_enabled = is_autostart_enabled();
    if currently_enabled {
        let _ = std::process::Command::new("reg")
            .args(["delete", "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run", "/v", "BornoFree", "/f"])
            .output();
        false
    } else {
        if let Ok(exe_path) = std::env::current_exe() {
            let path_str = exe_path.to_string_lossy();
            let _ = std::process::Command::new("reg")
                .args(["add", "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run", "/v", "BornoFree", "/t", "REG_SZ", "/d", &format!("\"{}\"", path_str), "/f"])
                .output();
        }
        true
    }
}

/// Shows popup menu when clicking the Mode selector on the floating bar.
unsafe fn show_mode_popup_menu(hwnd: isize, x: i32, y: i32) {
    let menu = CreatePopupMenu();
    let current_mode = crate::windows_hook::get_current_mode();

    let en_flag = if current_mode == InputMode::English { MF_CHECKED } else { MF_UNCHECKED };
    let uni_flag = if current_mode == InputMode::BornoUnicode { MF_CHECKED } else { MF_UNCHECKED };

    AppendMenuW(menu, MF_STRING | en_flag, ID_MENU_ENGLISH, to_wide("English (Ctrl+Alt+E)").as_ptr());
    AppendMenuW(menu, MF_STRING | uni_flag, ID_MENU_UNICODE, to_wide("বর্ণ ইউনিকোড (Ctrl+Alt+V)").as_ptr());

    SetForegroundWindow(hwnd);
    TrackPopupMenuEx(menu, TPM_RIGHTBUTTON, x, y, hwnd, std::ptr::null_mut());
    DestroyMenu(menu);
}

/// Shows Settings / Theme popup menu when clicking the Gear button.
unsafe fn show_settings_popup_menu(hwnd: isize, x: i32, y: i32) {
    let menu = CreatePopupMenu();
    let is_dark = IS_DARK_THEME.load(Ordering::SeqCst);
    let topmost = IS_TOPMOST.load(Ordering::SeqCst);

    let theme_label = if is_dark {
        "☀️ Switch to Light Theme (লাইট থিম)"
    } else {
        "🌙 Switch to Dark Theme (ডার্ক থিম)"
    };
    AppendMenuW(menu, MF_STRING, ID_MENU_TOGGLE_THEME, to_wide(theme_label).as_ptr());

    let top_flag = if topmost { MF_CHECKED } else { MF_UNCHECKED };
    AppendMenuW(menu, MF_STRING | top_flag, ID_MENU_TOGGLE_TOPMOST, to_wide("Always on Top (উপরে পিন রাখা)").as_ptr());

    let auto_flag = if is_autostart_enabled() { MF_CHECKED } else { MF_UNCHECKED };
    AppendMenuW(menu, MF_STRING | auto_flag, ID_MENU_TOGGLE_AUTOSTART, to_wide("Start with Windows (পিসি অন হলে চালু)").as_ptr());

    AppendMenuW(menu, MF_SEPARATOR, 0, std::ptr::null());
    AppendMenuW(menu, MF_STRING, ID_MENU_ABOUT, to_wide("About BornoFree (বর্ণফ্রি পরিচিতি)").as_ptr());

    SetForegroundWindow(hwnd);
    TrackPopupMenuEx(menu, TPM_RIGHTBUTTON, x, y, hwnd, std::ptr::null_mut());
    DestroyMenu(menu);
}

/// Shows full context menu when right-clicking the System Tray icon.
unsafe fn show_tray_context_menu(hwnd: isize, x: i32, y: i32) {
    let menu = CreatePopupMenu();
    let current_mode = crate::windows_hook::get_current_mode();
    let is_visible = IS_BAR_VISIBLE.load(Ordering::SeqCst);
    let is_dark = IS_DARK_THEME.load(Ordering::SeqCst);

    let en_flag = if current_mode == InputMode::English { MF_CHECKED } else { MF_UNCHECKED };
    let uni_flag = if current_mode == InputMode::BornoUnicode { MF_CHECKED } else { MF_UNCHECKED };

    AppendMenuW(menu, MF_STRING | en_flag, ID_MENU_ENGLISH, to_wide("English (Ctrl+Alt+E)").as_ptr());
    AppendMenuW(menu, MF_STRING | uni_flag, ID_MENU_UNICODE, to_wide("বর্ণ ইউনিকোড (Ctrl+Alt+V)").as_ptr());
    AppendMenuW(menu, MF_SEPARATOR, 0, std::ptr::null());

    let toggle_title = if is_visible {
        "Hide Floating Toolbar (হাইড বার)"
    } else {
        "Show Floating Toolbar (শো বার)"
    };
    AppendMenuW(menu, MF_STRING, ID_MENU_TOGGLE_BAR, to_wide(toggle_title).as_ptr());

    let theme_label = if is_dark {
        "☀️ Light Theme (লাইট থিম)"
    } else {
        "🌙 Dark Theme (ডার্ক থিম)"
    };
    AppendMenuW(menu, MF_STRING, ID_MENU_TOGGLE_THEME, to_wide(theme_label).as_ptr());

    let auto_flag = if is_autostart_enabled() { MF_CHECKED } else { MF_UNCHECKED };
    AppendMenuW(menu, MF_STRING | auto_flag, ID_MENU_TOGGLE_AUTOSTART, to_wide("Start with Windows (পিসি অন হলে চালু)").as_ptr());

    AppendMenuW(menu, MF_SEPARATOR, 0, std::ptr::null());
    AppendMenuW(menu, MF_STRING, ID_MENU_ABOUT, to_wide("About BornoFree").as_ptr());
    AppendMenuW(menu, MF_SEPARATOR, 0, std::ptr::null());
    AppendMenuW(menu, MF_STRING, ID_MENU_EXIT, to_wide("Exit BornoFree (প্রস্থান)").as_ptr());

    SetForegroundWindow(hwnd);
    TrackPopupMenuEx(menu, TPM_RIGHTBUTTON, x, y, hwnd, std::ptr::null_mut());
    DestroyMenu(menu);
}

/// Registers the authentic BornoFree icon in the Windows Taskbar Notification Area (SysTray).
unsafe fn setup_tray_icon(hwnd: isize) {
    let mode = crate::windows_hook::get_current_mode();
    let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
    nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = 1;
    nid.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
    nid.uCallbackMessage = WM_APP_TRAY;
    nid.hIcon = get_or_load_tray_icon();

    let tip = match mode {
        InputMode::English => to_wide("BornoFree: English (Ctrl+Alt+E)"),
        InputMode::BornoUnicode => to_wide("BornoFree: বর্ণ ইউনিকোড (Ctrl+Alt+V)"),
        _ => to_wide("BornoFree"),
    };
    let len = tip.len().min(127);
    nid.szTip[..len].copy_from_slice(&tip[..len]);

    Shell_NotifyIconW(NIM_ADD, &nid);
}

/// Dynamically updates the System Tray icon and tooltip to reflect the active mode.
unsafe fn update_tray_icon_and_tooltip(hwnd: isize) {
    let mode = crate::windows_hook::get_current_mode();
    let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
    nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = 1;
    nid.uFlags = NIF_ICON | NIF_TIP;
    nid.hIcon = get_or_load_tray_icon();

    let tip = match mode {
        InputMode::English => to_wide("BornoFree: English (Ctrl+Alt+E)"),
        InputMode::BornoUnicode => to_wide("BornoFree: বর্ণ ইউনিকোড (Ctrl+Alt+V)"),
        _ => to_wide("BornoFree"),
    };
    let len = tip.len().min(127);
    nid.szTip[..len].copy_from_slice(&tip[..len]);

    Shell_NotifyIconW(NIM_MODIFY, &nid);
}

/// Removes the System Tray icon when BornoFree exits.
unsafe fn remove_tray_icon(hwnd: isize) {
    let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
    nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = 1;
    Shell_NotifyIconW(NIM_DELETE, &nid);
}

/// Initializes and launches the BornoFree Floating Top Bar and native message loop.
pub fn launch_gui_and_message_loop() {
    unsafe {
        let class_name = to_wide("BornoFreeFloatingBar");
        let default_hicon = get_or_load_tray_icon();

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: 0,
            lpfnWndProc: bar_wnd_proc,
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: 0,
            hIcon: default_hicon,
            hCursor: LoadCursorW(0, 32512 as *const u16),
            hbrBackground: 0,
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name.as_ptr(),
            hIconSm: default_hicon,
        };

        let _atom = RegisterClassExW(&wc);

        // Modern Pill Toolbar Dimensions: 420px width, 42px height!
        let bar_width = 420;
        let bar_height = 42;

        // Position: Top Center of primary display with 10px top floating margin
        let screen_width = GetSystemMetrics(0); // SM_CXSCREEN
        let start_x = (screen_width - bar_width) / 2;
        let start_y = 10;

        let window_title = to_wide("BornoFree");
        let hwnd = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
            class_name.as_ptr(),
            window_title.as_ptr(),
            WS_POPUP | WS_VISIBLE,
            start_x,
            start_y,
            bar_width,
            bar_height,
            0,
            0,
            0,
            std::ptr::null_mut(),
        );

        if hwnd == 0 {
            return;
        }

        // Apply smooth rounded pill clipping region to the native window
        let rgn = CreateRoundRectRgn(0, 0, bar_width + 1, bar_height + 1, 18, 18);
        SetWindowRgn(hwnd, rgn, 1);

        GUI_HWND.store(hwnd, Ordering::SeqCst);
        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);

        // Standard Win32 Message Pump
        let mut msg = [0u8; 48];
        extern "system" {
            fn GetMessageW(lpMsg: *mut u8, hWnd: isize, wMsgFilterMin: u32, wMsgFilterMax: u32) -> i32;
            fn TranslateMessage(lpMsg: *const u8) -> i32;
            fn DispatchMessageW(lpMsg: *const u8) -> isize;
        }

        while GetMessageW(msg.as_mut_ptr(), 0, 0, 0) > 0 {
            TranslateMessage(msg.as_ptr());
            DispatchMessageW(msg.as_ptr());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gui_window_creation() {
        unsafe {
            extern "system" {
                fn GetLastError() -> u32;
            }

            let class_name = to_wide("TestBornoClass");
            let icon = get_or_load_tray_icon();
            assert_ne!(icon, 0, "Icon handle must not be 0");

            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: 0,
                lpfnWndProc: bar_wnd_proc,
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: 0,
                hIcon: icon,
                hCursor: LoadCursorW(0, 32512 as *const u16),
                hbrBackground: 0,
                lpszMenuName: std::ptr::null(),
                lpszClassName: class_name.as_ptr(),
                hIconSm: icon,
            };

            let atom = RegisterClassExW(&wc);
            let err1 = GetLastError();
            println!("RegisterClassExW atom: {}, last_err: {}", atom, err1);
            assert_ne!(atom, 0, "RegisterClassExW failed: {}", err1);

            let hwnd = CreateWindowExW(
                WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
                class_name.as_ptr(),
                to_wide("TestTitle").as_ptr(),
                WS_POPUP | WS_VISIBLE,
                100, 100, 420, 42,
                0, 0, 0, std::ptr::null_mut(),
            );
            let err2 = GetLastError();
            println!("CreateWindowExW hwnd: {}, last_err: {}", hwnd, err2);
            assert_ne!(hwnd, 0, "CreateWindowExW failed: {}", err2);

            DestroyWindow(hwnd);
        }
    }
}
