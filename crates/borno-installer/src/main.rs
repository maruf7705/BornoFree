#![windows_subsystem = "windows"]

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicIsize, AtomicU32, Ordering};

const PAYLOAD_EXE: &[u8] = include_bytes!("../../../BornoFree.exe");
const BOOK_LOGO_PNG: &[u8] = include_bytes!("../../borno-cli/icons/bornofree_book.png");

// Win32 Constants
const WS_OVERLAPPED: u32 = 0x00000000;
const WS_CAPTION: u32 = 0x00C00000;
const WS_SYSMENU: u32 = 0x00080000;
const WS_MINIMIZEBOX: u32 = 0x00020000;
const WS_VISIBLE: u32 = 0x10000000;
const WS_CHILD: u32 = 0x40000000;
const WS_TABSTOP: u32 = 0x00010000;
const BS_DEFPUSHBUTTON: u32 = 0x00000001;
const BS_AUTOCHECKBOX: u32 = 0x00000003;
const BST_CHECKED: usize = 1;

const WM_CREATE: u32 = 0x0001;
const WM_DESTROY: u32 = 0x0002;
const WM_PAINT: u32 = 0x000F;
const WM_COMMAND: u32 = 0x0111;
const WM_TIMER: u32 = 0x0113;
const WM_SETFONT: u32 = 0x0030;
const BM_GETCHECK: u32 = 0x00F0;
const BM_SETCHECK: u32 = 0x00F1;
const PBM_SETRANGE32: u32 = 0x0400 + 6;
const PBM_SETPOS: u32 = 0x0400 + 2;

const SRCCOPY: u32 = 0x00CC0020;
const TRANSPARENT: i32 = 1;
const PS_SOLID: i32 = 0;
const DI_NORMAL: u32 = 0x0003;
const MB_YESNO: u32 = 0x00000004;
const MB_ICONQUESTION: u32 = 0x00000020;
const IDYES: i32 = 6;

// Control IDs
const IDC_BTN_BACK: usize = 101;
const IDC_BTN_NEXT: usize = 102;
const IDC_BTN_CANCEL: usize = 103;
const IDC_CHK_DESKTOP: usize = 201;
const IDC_CHK_STARTMENU: usize = 202;
const IDC_CHK_AUTOSTART: usize = 203;
const IDC_CHK_LAUNCH: usize = 204;
const IDC_PROGRESS: usize = 301;

// Wizard Pages
#[derive(Clone, Copy, PartialEq, Eq)]
enum WizardPage {
    Welcome = 0,
    Options = 1,
    Installing = 2,
    Finished = 3,
}

static CURRENT_PAGE: AtomicU32 = AtomicU32::new(WizardPage::Welcome as u32);
static PROGRESS_VALUE: AtomicU32 = AtomicU32::new(0);
static HICON_LOGO: AtomicIsize = AtomicIsize::new(0);

#[repr(C)]
struct RECT {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[repr(C)]
struct PAINTSTRUCT {
    hdc: isize,
    f_erase: i32,
    rc_paint: RECT,
    f_restore: i32,
    f_inc_update: i32,
    rgb_reserved: [u8; 32],
}

#[repr(C)]
struct WNDCLASSEXW {
    cb_size: u32,
    style: u32,
    lpfn_wnd_proc: unsafe extern "system" fn(isize, u32, usize, isize) -> isize,
    cb_cls_extra: i32,
    cb_wnd_extra: i32,
    h_instance: isize,
    h_icon: isize,
    h_cursor: isize,
    hbr_background: isize,
    lpsz_menu_name: *const u16,
    lpsz_class_name: *const u16,
    h_icon_sm: isize,
}

#[link(name = "user32")]
#[link(name = "gdi32")]
#[link(name = "shell32")]
#[link(name = "comctl32")]
extern "system" {
    fn RegisterClassExW(lpwcx: *const WNDCLASSEXW) -> u16;
    fn CreateWindowExW(
        dw_ex_style: u32,
        lp_class_name: *const u16,
        lp_window_name: *const u16,
        dw_style: u32,
        x: i32,
        y: i32,
        n_width: i32,
        n_height: i32,
        h_wnd_parent: isize,
        h_menu: isize,
        h_instance: isize,
        lp_param: *mut std::ffi::c_void,
    ) -> isize;
    fn ShowWindow(h_wnd: isize, n_cmd_show: i32) -> i32;
    fn UpdateWindow(h_wnd: isize) -> i32;
    fn DestroyWindow(h_wnd: isize) -> i32;
    fn DefWindowProcW(h_wnd: isize, msg: u32, w_param: usize, l_param: isize) -> isize;
    fn PostQuitMessage(n_exit_code: i32);
    fn GetMessageW(lp_msg: *mut u8, h_wnd: isize, w_msg_filter_min: u32, w_msg_filter_max: u32) -> i32;
    fn TranslateMessage(lp_msg: *const u8) -> i32;
    fn DispatchMessageW(lp_msg: *const u8) -> isize;
    fn BeginPaint(h_wnd: isize, lp_paint: *mut PAINTSTRUCT) -> isize;
    fn EndPaint(h_wnd: isize, lp_paint: *const PAINTSTRUCT) -> i32;
    fn InvalidateRect(h_wnd: isize, lp_rect: *const RECT, b_erase: i32) -> i32;
    fn GetClientRect(h_wnd: isize, lp_rect: *mut RECT) -> i32;
    fn CreateCompatibleDC(hdc: isize) -> isize;
    fn CreateCompatibleBitmap(hdc: isize, cx: i32, cy: i32) -> isize;
    fn SelectObject(hdc: isize, h: isize) -> isize;
    fn DeleteObject(ho: isize) -> i32;
    fn DeleteDC(hdc: isize) -> i32;
    fn BitBlt(
        hdc_dest: isize,
        n_x_dest: i32,
        n_y_dest: i32,
        n_width: i32,
        n_height: i32,
        hdc_src: isize,
        n_x_src: i32,
        n_y_src: i32,
        dw_rop: u32,
    ) -> i32;
    fn CreateSolidBrush(color: u32) -> isize;
    fn CreatePen(i_style: i32, c_width: i32, color: u32) -> isize;
    fn FillRect(hdc: isize, lprc: *const RECT, hbr: isize) -> i32;
    fn MoveToEx(hdc: isize, x: i32, y: i32, lppt: *mut std::ffi::c_void) -> i32;
    fn LineTo(hdc: isize, x: i32, y: i32) -> i32;
    fn RoundRect(hdc: isize, left: i32, top: i32, right: i32, bottom: i32, width: i32, height: i32) -> i32;
    fn SetBkMode(hdc: isize, mode: i32) -> i32;
    fn SetTextColor(hdc: isize, color: u32) -> u32;
    fn CreateFontW(
        c_height: i32,
        c_width: i32,
        c_escapement: i32,
        c_orientation: i32,
        c_weight: i32,
        b_italic: u32,
        b_underline: u32,
        b_strike_out: u32,
        i_char_set: u32,
        i_out_precision: u32,
        i_clip_precision: u32,
        i_quality: u32,
        i_pitch_and_family: u32,
        psz_face_name: *const u16,
    ) -> isize;
    fn DrawTextW(hdc: isize, lpch_text: *const u16, cch_text: i32, lprc: *mut RECT, format: u32) -> i32;
    fn LoadCursorW(h_instance: isize, lp_cursor_name: *const u16) -> isize;
    fn CreateIconFromResourceEx(
        pb_icon_bits: *const u8,
        cb_icon_bits: u32,
        f_icon: i32,
        dw_version: u32,
        cx_desired: i32,
        cy_desired: i32,
        u_flags: u32,
    ) -> isize;
    fn DrawIconEx(
        hdc: isize,
        x_left: i32,
        y_top: i32,
        h_icon: isize,
        cx_width: i32,
        cy_width: i32,
        istep_if_ani_cur: u32,
        hbr_flicker_free_draw: isize,
        di_flags: u32,
    ) -> i32;
    fn SendMessageW(h_wnd: isize, msg: u32, w_param: usize, l_param: isize) -> isize;
    fn SetWindowTextW(h_wnd: isize, lp_string: *const u16) -> i32;
    fn EnableWindow(h_wnd: isize, b_enable: i32) -> i32;
    fn GetSystemMetrics(n_index: i32) -> i32;
    fn SetTimer(h_wnd: isize, n_id_event: usize, u_elapse: u32, lp_timer_func: *mut std::ffi::c_void) -> usize;
    fn KillTimer(h_wnd: isize, u_id_event: usize) -> i32;
    fn MessageBoxW(h_wnd: isize, lp_text: *const u16, lp_caption: *const u16, u_type: u32) -> i32;
}

fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

fn rgb(r: u8, g: u8, b: u8) -> u32 {
    (r as u32) | ((g as u32) << 8) | ((b as u32) << 16)
}

fn get_logo_icon() -> isize {
    let mut h = HICON_LOGO.load(Ordering::SeqCst);
    if h == 0 {
        unsafe {
            h = CreateIconFromResourceEx(
                BOOK_LOGO_PNG.as_ptr(),
                BOOK_LOGO_PNG.len() as u32,
                1,
                0x00030000,
                64,
                64,
                0,
            );
        }
        HICON_LOGO.store(h, Ordering::SeqCst);
    }
    h
}

// Child control handles
static HWND_BTN_BACK: AtomicIsize = AtomicIsize::new(0);
static HWND_BTN_NEXT: AtomicIsize = AtomicIsize::new(0);
static HWND_BTN_CANCEL: AtomicIsize = AtomicIsize::new(0);
static HWND_CHK_DESKTOP: AtomicIsize = AtomicIsize::new(0);
static HWND_CHK_STARTMENU: AtomicIsize = AtomicIsize::new(0);
static HWND_CHK_AUTOSTART: AtomicIsize = AtomicIsize::new(0);
static HWND_CHK_LAUNCH: AtomicIsize = AtomicIsize::new(0);
static HWND_PROGRESS: AtomicIsize = AtomicIsize::new(0);

fn update_control_visibility(page: WizardPage) {
    let show_options = if page == WizardPage::Options { 1 } else { 0 };
    let show_installing = if page == WizardPage::Installing { 1 } else { 0 };
    let show_finished = if page == WizardPage::Finished { 1 } else { 0 };

    unsafe {
        ShowWindow(HWND_CHK_DESKTOP.load(Ordering::SeqCst), if show_options == 1 { 5 } else { 0 });
        ShowWindow(HWND_CHK_STARTMENU.load(Ordering::SeqCst), if show_options == 1 { 5 } else { 0 });
        ShowWindow(HWND_CHK_AUTOSTART.load(Ordering::SeqCst), if show_options == 1 { 5 } else { 0 });

        ShowWindow(HWND_PROGRESS.load(Ordering::SeqCst), if show_installing == 1 { 5 } else { 0 });
        ShowWindow(HWND_CHK_LAUNCH.load(Ordering::SeqCst), if show_finished == 1 { 5 } else { 0 });

        let btn_back = HWND_BTN_BACK.load(Ordering::SeqCst);
        let btn_next = HWND_BTN_NEXT.load(Ordering::SeqCst);
        let btn_cancel = HWND_BTN_CANCEL.load(Ordering::SeqCst);

        match page {
            WizardPage::Welcome => {
                EnableWindow(btn_back, 0);
                EnableWindow(btn_next, 1);
                EnableWindow(btn_cancel, 1);
                SetWindowTextW(btn_next, to_wide("Next >").as_ptr());
            }
            WizardPage::Options => {
                EnableWindow(btn_back, 1);
                EnableWindow(btn_next, 1);
                EnableWindow(btn_cancel, 1);
                SetWindowTextW(btn_next, to_wide("Install").as_ptr());
            }
            WizardPage::Installing => {
                EnableWindow(btn_back, 0);
                EnableWindow(btn_next, 0);
                EnableWindow(btn_cancel, 0);
            }
            WizardPage::Finished => {
                EnableWindow(btn_back, 0);
                EnableWindow(btn_next, 1);
                EnableWindow(btn_cancel, 0);
                SetWindowTextW(btn_next, to_wide("Finish").as_ptr());
            }
        }
    }
}

fn get_install_dir() -> PathBuf {
    let local_app_data = std::env::var("LOCALAPPDATA")
        .unwrap_or_else(|_| "C:\\Users\\Default\\AppData\\Local".to_string());
    PathBuf::from(local_app_data).join("Programs").join("BornoFree")
}

fn perform_installation(chk_desktop: bool, chk_startmenu: bool, chk_autostart: bool) {
    let install_dir = get_install_dir();
    let _ = std::fs::create_dir_all(&install_dir);

    // 1. Stop any running BornoFree process
    let _ = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", "Stop-Process -Name 'BornoFree' -Force -ErrorAction SilentlyContinue"])
        .output();

    // 2. Write payload BornoFree.exe
    let exe_path = install_dir.join("BornoFree.exe");
    let _ = std::fs::write(&exe_path, PAYLOAD_EXE);

    let exe_str = exe_path.to_string_lossy().to_string();
    let dir_str = install_dir.to_string_lossy().to_string();

    // 3. Shortcuts creation via PowerShell WScript.Shell
    let mut ps_script = format!(
        "$ws = New-Object -ComObject WScript.Shell; \
        $exe = '{exe}'; \
        $dir = '{dir}'; ",
        exe = exe_str.replace('\'', "''"),
        dir = dir_str.replace('\'', "''")
    );

    if chk_desktop {
        ps_script.push_str(
            "$desktop = [Environment]::GetFolderPath('Desktop'); \
            $sc1 = $ws.CreateShortcut(\"$desktop\\BornoFree.lnk\"); \
            $sc1.TargetPath = $exe; \
            $sc1.WorkingDirectory = $dir; \
            $sc1.IconLocation = \"$exe,0\"; \
            $sc1.Description = 'BornoFree - Modern Bengali Typing Engine'; \
            $sc1.Save(); "
        );
    }

    if chk_startmenu {
        ps_script.push_str(
            "$startMenu = [Environment]::GetFolderPath('StartMenu') + '\\Programs'; \
            $sc2 = $ws.CreateShortcut(\"$startMenu\\BornoFree.lnk\"); \
            $sc2.TargetPath = $exe; \
            $sc2.WorkingDirectory = $dir; \
            $sc2.IconLocation = \"$exe,0\"; \
            $sc2.Description = 'BornoFree - Modern Bengali Typing Engine'; \
            $sc2.Save(); "
        );
    }

    if chk_autostart {
        ps_script.push_str(
            "Set-ItemProperty -Path 'HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Run' -Name 'BornoFree' -Value \"`\"$exe`\"\"; "
        );
    }

    // Register in Add/Remove Programs
    ps_script.push_str(
        "$regPath = 'HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\BornoFree'; \
        if (-not (Test-Path $regPath)) { New-Item -Path $regPath -Force | Out-Null }; \
        Set-ItemProperty -Path $regPath -Name 'DisplayName' -Value 'BornoFree (বর্ণফ্রি)'; \
        Set-ItemProperty -Path $regPath -Name 'DisplayVersion' -Value '1.0.0'; \
        Set-ItemProperty -Path $regPath -Name 'Publisher' -Value 'BornoFree Open Source Project'; \
        Set-ItemProperty -Path $regPath -Name 'DisplayIcon' -Value $exe; \
        Set-ItemProperty -Path $regPath -Name 'InstallLocation' -Value $dir; "
    );

    let _ = std::process::Command::new("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &ps_script])
        .output();
}

unsafe extern "system" fn wizard_wnd_proc(hwnd: isize, msg: u32, w_param: usize, l_param: isize) -> isize {
    match msg {
        WM_CREATE => {
            let font_name = to_wide("Segoe UI");
            let ui_font = CreateFontW(14, 0, 0, 0, 400, 0, 0, 0, 1, 0, 0, 5, 0, font_name.as_ptr());

            // Navigation Buttons
            let btn_class = to_wide("BUTTON");
            let btn_back = CreateWindowExW(
                0,
                btn_class.as_ptr(),
                to_wide("< Back").as_ptr(),
                WS_CHILD | WS_VISIBLE | WS_TABSTOP,
                245, 320, 75, 25,
                hwnd, IDC_BTN_BACK as isize, 0, std::ptr::null_mut(),
            );
            SendMessageW(btn_back, WM_SETFONT, ui_font as usize, 1);
            HWND_BTN_BACK.store(btn_back, Ordering::SeqCst);

            let btn_next = CreateWindowExW(
                0,
                btn_class.as_ptr(),
                to_wide("Next >").as_ptr(),
                WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_DEFPUSHBUTTON,
                328, 320, 75, 25,
                hwnd, IDC_BTN_NEXT as isize, 0, std::ptr::null_mut(),
            );
            SendMessageW(btn_next, WM_SETFONT, ui_font as usize, 1);
            HWND_BTN_NEXT.store(btn_next, Ordering::SeqCst);

            let btn_cancel = CreateWindowExW(
                0,
                btn_class.as_ptr(),
                to_wide("Cancel").as_ptr(),
                WS_CHILD | WS_VISIBLE | WS_TABSTOP,
                411, 320, 75, 25,
                hwnd, IDC_BTN_CANCEL as isize, 0, std::ptr::null_mut(),
            );
            SendMessageW(btn_cancel, WM_SETFONT, ui_font as usize, 1);
            HWND_BTN_CANCEL.store(btn_cancel, Ordering::SeqCst);

            // Checkboxes for Page 1 (Options)
            let chk_desktop = CreateWindowExW(
                0,
                btn_class.as_ptr(),
                to_wide("Create a Desktop shortcut").as_ptr(),
                WS_CHILD | BS_AUTOCHECKBOX,
                185, 120, 300, 22,
                hwnd, IDC_CHK_DESKTOP as isize, 0, std::ptr::null_mut(),
            );
            SendMessageW(chk_desktop, WM_SETFONT, ui_font as usize, 1);
            SendMessageW(chk_desktop, BM_SETCHECK, BST_CHECKED, 0);
            HWND_CHK_DESKTOP.store(chk_desktop, Ordering::SeqCst);

            let chk_startmenu = CreateWindowExW(
                0,
                btn_class.as_ptr(),
                to_wide("Create a Start Menu shortcut").as_ptr(),
                WS_CHILD | BS_AUTOCHECKBOX,
                185, 150, 300, 22,
                hwnd, IDC_CHK_STARTMENU as isize, 0, std::ptr::null_mut(),
            );
            SendMessageW(chk_startmenu, WM_SETFONT, ui_font as usize, 1);
            SendMessageW(chk_startmenu, BM_SETCHECK, BST_CHECKED, 0);
            HWND_CHK_STARTMENU.store(chk_startmenu, Ordering::SeqCst);

            let chk_autostart = CreateWindowExW(
                0,
                btn_class.as_ptr(),
                to_wide("Start BornoFree automatically when Windows boots").as_ptr(),
                WS_CHILD | BS_AUTOCHECKBOX,
                185, 180, 310, 22,
                hwnd, IDC_CHK_AUTOSTART as isize, 0, std::ptr::null_mut(),
            );
            SendMessageW(chk_autostart, WM_SETFONT, ui_font as usize, 1);
            SendMessageW(chk_autostart, BM_SETCHECK, BST_CHECKED, 0);
            HWND_CHK_AUTOSTART.store(chk_autostart, Ordering::SeqCst);

            // Progress Bar for Page 2 (Installing)
            let pbm_class = to_wide("msctls_progress32");
            let progress = CreateWindowExW(
                0,
                pbm_class.as_ptr(),
                std::ptr::null(),
                WS_CHILD,
                185, 140, 290, 20,
                hwnd, IDC_PROGRESS as isize, 0, std::ptr::null_mut(),
            );
            SendMessageW(progress, PBM_SETRANGE32, 0, 100);
            HWND_PROGRESS.store(progress, Ordering::SeqCst);

            // Launch Checkbox for Page 3 (Finished)
            let chk_launch = CreateWindowExW(
                0,
                btn_class.as_ptr(),
                to_wide("Launch BornoFree now").as_ptr(),
                WS_CHILD | BS_AUTOCHECKBOX,
                185, 160, 300, 22,
                hwnd, IDC_CHK_LAUNCH as isize, 0, std::ptr::null_mut(),
            );
            SendMessageW(chk_launch, WM_SETFONT, ui_font as usize, 1);
            SendMessageW(chk_launch, BM_SETCHECK, BST_CHECKED, 0);
            HWND_CHK_LAUNCH.store(chk_launch, Ordering::SeqCst);

            update_control_visibility(WizardPage::Welcome);
            0
        }
        WM_COMMAND => {
            let id = w_param & 0xFFFF;
            let current = match CURRENT_PAGE.load(Ordering::SeqCst) {
                0 => WizardPage::Welcome,
                1 => WizardPage::Options,
                2 => WizardPage::Installing,
                _ => WizardPage::Finished,
            };

            match id {
                IDC_BTN_BACK => {
                    if current == WizardPage::Options {
                        CURRENT_PAGE.store(WizardPage::Welcome as u32, Ordering::SeqCst);
                        update_control_visibility(WizardPage::Welcome);
                        InvalidateRect(hwnd, std::ptr::null(), 1);
                    }
                }
                IDC_BTN_NEXT => {
                    match current {
                        WizardPage::Welcome => {
                            CURRENT_PAGE.store(WizardPage::Options as u32, Ordering::SeqCst);
                            update_control_visibility(WizardPage::Options);
                            InvalidateRect(hwnd, std::ptr::null(), 1);
                        }
                        WizardPage::Options => {
                            CURRENT_PAGE.store(WizardPage::Installing as u32, Ordering::SeqCst);
                            PROGRESS_VALUE.store(0, Ordering::SeqCst);
                            update_control_visibility(WizardPage::Installing);
                            InvalidateRect(hwnd, std::ptr::null(), 1);

                            // Start 50ms timer for smooth progress animation & installation
                            SetTimer(hwnd, 1, 50, std::ptr::null_mut());
                        }
                        WizardPage::Finished => {
                            let launch_hwnd = HWND_CHK_LAUNCH.load(Ordering::SeqCst);
                            let should_launch = SendMessageW(launch_hwnd, BM_GETCHECK, 0, 0) == BST_CHECKED as isize;
                            if should_launch {
                                let exe_path = get_install_dir().join("BornoFree.exe");
                                let _ = std::process::Command::new(exe_path).spawn();
                            }
                            DestroyWindow(hwnd);
                        }
                        _ => {}
                    }
                }
                IDC_BTN_CANCEL => {
                    let confirm = MessageBoxW(
                        hwnd,
                        to_wide("Are you sure you want to cancel BornoFree setup?").as_ptr(),
                        to_wide("Cancel Setup").as_ptr(),
                        MB_YESNO | MB_ICONQUESTION,
                    );
                    if confirm == IDYES {
                        DestroyWindow(hwnd);
                    }
                }
                _ => {}
            }
            0
        }
        WM_TIMER => {
            let mut val = PROGRESS_VALUE.load(Ordering::SeqCst);
            if val < 100 {
                val += 5;
                PROGRESS_VALUE.store(val, Ordering::SeqCst);
                let p_hwnd = HWND_PROGRESS.load(Ordering::SeqCst);
                SendMessageW(p_hwnd, PBM_SETPOS, val as usize, 0);

                if val == 50 {
                    // Perform real installation at 50%
                    let chk_desktop = SendMessageW(HWND_CHK_DESKTOP.load(Ordering::SeqCst), BM_GETCHECK, 0, 0) == BST_CHECKED as isize;
                    let chk_startmenu = SendMessageW(HWND_CHK_STARTMENU.load(Ordering::SeqCst), BM_GETCHECK, 0, 0) == BST_CHECKED as isize;
                    let chk_autostart = SendMessageW(HWND_CHK_AUTOSTART.load(Ordering::SeqCst), BM_GETCHECK, 0, 0) == BST_CHECKED as isize;
                    perform_installation(chk_desktop, chk_startmenu, chk_autostart);
                }
            } else {
                KillTimer(hwnd, 1);
                CURRENT_PAGE.store(WizardPage::Finished as u32, Ordering::SeqCst);
                update_control_visibility(WizardPage::Finished);
                InvalidateRect(hwnd, std::ptr::null(), 1);
            }
            0
        }
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = std::mem::zeroed();
            let screen_dc = BeginPaint(hwnd, &mut ps);

            let mut client_rc: RECT = std::mem::zeroed();
            GetClientRect(hwnd, &mut client_rc);
            let width = client_rc.right - client_rc.left;
            let height = client_rc.bottom - client_rc.top;

            // Double buffering memory DC
            let mem_dc = CreateCompatibleDC(screen_dc);
            let mem_bmp = CreateCompatibleBitmap(screen_dc, width, height);
            let old_bmp = SelectObject(mem_dc, mem_bmp);

            // Fill background white
            let white_brush = CreateSolidBrush(rgb(255, 255, 255));
            FillRect(mem_dc, &client_rc, white_brush);
            DeleteObject(white_brush);

            // Left Sidebar Banner: Width 164px, Height 308px
            let banner_rc = RECT { left: 0, top: 0, right: 164, bottom: 308 };
            let banner_brush = CreateSolidBrush(rgb(22, 27, 34)); // Dark Pro Slate
            FillRect(mem_dc, &banner_rc, banner_brush);
            DeleteObject(banner_brush);

            // Left Sidebar: White Logo Card at X=24, Y=25, 116x116
            let card_brush = CreateSolidBrush(rgb(255, 255, 255));
            let card_pen = CreatePen(PS_SOLID, 1, rgb(48, 54, 61));
            let old_card_brush = SelectObject(mem_dc, card_brush);
            let old_card_pen = SelectObject(mem_dc, card_pen);
            RoundRect(mem_dc, 24, 25, 140, 141, 16, 16);
            SelectObject(mem_dc, old_card_brush);
            SelectObject(mem_dc, old_card_pen);
            DeleteObject(card_brush);
            DeleteObject(card_pen);

            // Draw 3D Blue Book Logo inside the Card
            let h_logo = get_logo_icon();
            if h_logo != 0 {
                DrawIconEx(mem_dc, 50, 51, h_logo, 64, 64, 0, 0, DI_NORMAL);
            }

            // Left Sidebar Text
            let font_name = to_wide("Segoe UI");
            let brand_font = CreateFontW(18, 0, 0, 0, 700, 0, 0, 0, 1, 0, 0, 5, 0, font_name.as_ptr());
            let sub_font = CreateFontW(13, 0, 0, 0, 400, 0, 0, 0, 1, 0, 0, 5, 0, font_name.as_ptr());

            SetBkMode(mem_dc, TRANSPARENT);
            SetTextColor(mem_dc, rgb(255, 255, 255));

            let old_font = SelectObject(mem_dc, brand_font);
            let mut brand_rc = RECT { left: 10, top: 155, right: 154, bottom: 180 };
            DrawTextW(mem_dc, to_wide("BornoFree").as_ptr(), -1, &mut brand_rc, 0x00000001); // DT_CENTER

            SelectObject(mem_dc, sub_font);
            SetTextColor(mem_dc, rgb(139, 148, 158));
            let mut bn_rc = RECT { left: 10, top: 180, right: 154, bottom: 205 };
            DrawTextW(mem_dc, to_wide("বর্ণফ্রি v1.0.0").as_ptr(), -1, &mut bn_rc, 0x00000001);

            let mut plat_rc = RECT { left: 10, top: 260, right: 154, bottom: 285 };
            DrawTextW(mem_dc, to_wide("Windows 10 / 11").as_ptr(), -1, &mut plat_rc, 0x00000001);

            DeleteObject(brand_font);
            DeleteObject(sub_font);

            // Bottom Separator Bar at Y = 308
            let sep_brush = CreateSolidBrush(rgb(243, 243, 243));
            let bottom_rc = RECT { left: 0, top: 308, right: width, bottom: height };
            FillRect(mem_dc, &bottom_rc, sep_brush);
            DeleteObject(sep_brush);

            let line_pen = CreatePen(PS_SOLID, 1, rgb(210, 210, 210));
            let old_line_pen = SelectObject(mem_dc, line_pen);
            MoveToEx(mem_dc, 0, 308, std::ptr::null_mut());
            LineTo(mem_dc, width, 308);
            SelectObject(mem_dc, old_line_pen);
            DeleteObject(line_pen);

            // Right Content Area (X: 180..485)
            let page = match CURRENT_PAGE.load(Ordering::SeqCst) {
                0 => WizardPage::Welcome,
                1 => WizardPage::Options,
                2 => WizardPage::Installing,
                _ => WizardPage::Finished,
            };

            let title_font = CreateFontW(19, 0, 0, 0, 700, 0, 0, 0, 1, 0, 0, 5, 0, font_name.as_ptr());
            let body_font = CreateFontW(14, 0, 0, 0, 400, 0, 0, 0, 1, 0, 0, 5, 0, font_name.as_ptr());

            SelectObject(mem_dc, title_font);
            SetTextColor(mem_dc, rgb(22, 27, 34));

            match page {
                WizardPage::Welcome => {
                    let mut t_rc = RECT { left: 185, top: 25, right: 485, bottom: 80 };
                    DrawTextW(mem_dc, to_wide("Welcome to the BornoFree Setup Wizard").as_ptr(), -1, &mut t_rc, 0x00000810); // DT_WORDBREAK

                    SelectObject(mem_dc, body_font);
                    SetTextColor(mem_dc, rgb(87, 96, 106));
                    let mut b_rc = RECT { left: 185, top: 95, right: 485, bottom: 270 };
                    let welcome_body = to_wide(
                        "The Setup Wizard will install BornoFree (Modern Bengali Typing Engine) on your computer.\n\n\
                        BornoFree provides 0-lag typing, modern floating pill toolbar, and seamless Borno Unicode & Phonetic modes.\n\n\
                        Click Next to continue or Cancel to exit the Setup Wizard."
                    );
                    DrawTextW(mem_dc, welcome_body.as_ptr(), -1, &mut b_rc, 0x00000810);
                }
                WizardPage::Options => {
                    let mut t_rc = RECT { left: 185, top: 25, right: 485, bottom: 65 };
                    DrawTextW(mem_dc, to_wide("Select Installation Options").as_ptr(), -1, &mut t_rc, 0x00000810);

                    SelectObject(mem_dc, body_font);
                    SetTextColor(mem_dc, rgb(87, 96, 106));
                    let mut b_rc = RECT { left: 185, top: 75, right: 485, bottom: 115 };
                    let opt_body = to_wide("Select the additional shortcuts and features you would like Setup to create:");
                    DrawTextW(mem_dc, opt_body.as_ptr(), -1, &mut b_rc, 0x00000810);
                }
                WizardPage::Installing => {
                    let mut t_rc = RECT { left: 185, top: 25, right: 485, bottom: 65 };
                    DrawTextW(mem_dc, to_wide("Installing BornoFree").as_ptr(), -1, &mut t_rc, 0x00000810);

                    SelectObject(mem_dc, body_font);
                    SetTextColor(mem_dc, rgb(87, 96, 106));
                    let mut b_rc = RECT { left: 185, top: 75, right: 485, bottom: 125 };
                    let inst_body = to_wide("Please wait while Setup extracts files, creates shortcuts, and registers components on your computer...");
                    DrawTextW(mem_dc, inst_body.as_ptr(), -1, &mut b_rc, 0x00000810);
                }
                WizardPage::Finished => {
                    let mut t_rc = RECT { left: 185, top: 25, right: 485, bottom: 85 };
                    DrawTextW(mem_dc, to_wide("Completing the BornoFree Setup Wizard").as_ptr(), -1, &mut t_rc, 0x00000810);

                    SelectObject(mem_dc, body_font);
                    SetTextColor(mem_dc, rgb(87, 96, 106));
                    let mut b_rc = RECT { left: 185, top: 95, right: 485, bottom: 155 };
                    let fin_body = to_wide(
                        "BornoFree has been successfully installed on your computer.\n\n\
                        Click Finish to exit Setup."
                    );
                    DrawTextW(mem_dc, fin_body.as_ptr(), -1, &mut b_rc, 0x00000010);
                }
            }

            DeleteObject(title_font);
            DeleteObject(body_font);
            SelectObject(mem_dc, old_font);

            // Copy to screen
            BitBlt(screen_dc, 0, 0, width, height, mem_dc, 0, 0, SRCCOPY);

            SelectObject(mem_dc, old_bmp);
            DeleteObject(mem_bmp);
            DeleteDC(mem_dc);

            EndPaint(hwnd, &ps);
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, w_param, l_param),
    }
}

fn main() {
    unsafe {
        let class_name = to_wide("BornoFreeSetupClass");
        let default_icon = get_logo_icon();

        let wc = WNDCLASSEXW {
            cb_size: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: 0,
            lpfn_wnd_proc: wizard_wnd_proc,
            cb_cls_extra: 0,
            cb_wnd_extra: 0,
            h_instance: 0,
            h_icon: default_icon,
            h_cursor: LoadCursorW(0, 32512 as *const u16),
            hbr_background: 0,
            lpsz_menu_name: std::ptr::null(),
            lpsz_class_name: class_name.as_ptr(),
            h_icon_sm: default_icon,
        };

        RegisterClassExW(&wc);

        let win_width = 512;
        let win_height = 395;
        let screen_width = GetSystemMetrics(0);
        let screen_height = GetSystemMetrics(1);
        let start_x = (screen_width - win_width) / 2;
        let start_y = (screen_height - win_height) / 2;

        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            to_wide("BornoFree Setup").as_ptr(),
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX | WS_VISIBLE,
            start_x,
            start_y,
            win_width,
            win_height,
            0,
            0,
            0,
            std::ptr::null_mut(),
        );

        if hwnd == 0 {
            return;
        }

        ShowWindow(hwnd, 5); // SW_SHOW
        UpdateWindow(hwnd);

        let mut msg = [0u8; 48];
        while GetMessageW(msg.as_mut_ptr(), 0, 0, 0) > 0 {
            TranslateMessage(msg.as_ptr());
            DispatchMessageW(msg.as_ptr());
        }
    }
}
