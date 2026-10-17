# BornoFree (বর্ণফ্রি)

High-Performance Bengali Typing Engine and Input Method Architecture for Windows

> **Notice:** BornoFree is an independent project and is not affiliated with Bijoy or Ananda Computers.

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%2010%20%7C%2011-success.svg)](https://github.com/maruf7705/BornoFree)
[![Language](https://img.shields.io/badge/language-Rust%202021-orange.svg)](https://www.rust-lang.org/)
[![Version](https://img.shields.io/badge/version-1.0.1-green.svg)](https://github.com/maruf7705/BornoFree/releases)

<p align="center">
  <img src="BornoFree Light and Dark Toolbars.png" alt="BornoFree Floating Toolbar in Light and Dark Modes" width="760"/>
</p>

## Overview

BornoFree is a high-performance, open-source Bengali input method and typing engine engineered in pure Rust. Built to provide a clean, modern alternative for desktop Bengali input, BornoFree eliminates typing latency, removes dependencies on legacy runtime frameworks such as .NET Framework 3.5, and provides native, reliable Bengali Unicode text input across modern desktop applications.

The application features a non-intrusive floating pill toolbar with integrated Light and Dark themes, zero-latency buffer transformation, full support for modern Bengali Unicode and phonetic typing, and an authentic Windows Setup Wizard for seamless deployment.

---

## Downloads and Binaries

Ready-to-use binaries for Windows 10 and Windows 11 (64-bit):

| Package | File | Description |
| :--- | :--- | :--- |
| Graphical Setup Wizard (Recommended) | [BornoFree-Setup.exe](dist/BornoFree-Setup.exe) | Complete Windows installer wizard with automated Desktop and Start Menu shortcuts |
| Standalone Portable Binary | [BornoFree.exe](dist/BornoFree.exe) | Single self-contained executable running directly without installation |
| Complete Distribution Archive | [BornoFree-v1.0.1-Windows-x64.zip](dist/BornoFree-v1.0.1-Windows-x64.zip) | Portable archive containing the application, installer, and uninstaller |
| GitHub Releases | [Release Assets](https://github.com/maruf7705/BornoFree/releases) | Official version-tagged downloads and cryptographic checksums |

---

## Core Capabilities

### Zero-Latency Input Pipeline
Implemented in Rust utilizing native Win32 Low-Level Keyboard Hooks (`WH_KEYBOARD_LL`). Keystroke processing and conjunct reordering execute in under one millisecond, eliminating dropped keystrokes and typing lag present in legacy software.

### Input Modes
* Borno Unicode (`Ctrl + Alt + V`): Outputs standard Bengali Unicode characters (`U+0980` through `U+09FF`) for modern web browsers, office suites, and operating system controls.
* Avro Phonetic: Transliterates Roman English text to Bengali phonetically.
* English Passthrough (`Ctrl + Alt + E` or `F12`): Bypasses Bengali transformation for standard Roman alphanumeric typing.

### Native Floating Interface
A minimalist floating pill toolbar with double-buffered GDI rendering:
* Dual Themes: One-click toggle between Pro Dark (`#161B22`) and Clean Light (`#FFFFFF`) themes via the toolbar Settings gear.
* Mode Selector: Visual badge indicator (`[A]` for English, `[অ]` for Bengali) with layout selection menu.
* Window Controls: Draggable across any monitor screen edge, minimizable to the Windows Taskbar Notification Area (System Tray).

### Automatic Startup Integration
Configured to start automatically on Windows boot via User Registry integration (`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`), ensuring typing shortcuts are active immediately upon user login. Auto-start can be toggled directly from the toolbar Settings menu.

---

## Keyboard Shortcuts

| Shortcut | Layout Mode | Target Environment |
| :--- | :--- | :--- |
| `Ctrl + Alt + V` | Borno Unicode (বর্ণ ইউনিকোড) | Web browsers, Microsoft Office, modern applications |
| `Ctrl + Alt + E` | English | Standard Latin text entry |
| `F12` | Quick Toggle | Switches between English and Bengali Unicode layout |

---

## Workspace Structure

```
BornoFree/
├── Cargo.toml                      # Workspace manifest
├── Cargo.lock                      # Dependency lockfile
├── LICENSE                         # MIT License
├── README.md                       # Repository documentation
├── BornoFree Blue Book Logo.png    # High-resolution vector brand assets
├── BornoFree Light and Dark Toolbars.png
├── dist/                           # Production distribution binaries
│   ├── BornoFree-Setup.exe         # Windows Graphical Setup Wizard
│   ├── BornoFree.exe               # Standalone portable binary
│   ├── BornoFree-v1.0.1-Windows-x64.zip
│   └── SHA256SUMS.txt              # SHA-256 verification hashes
├── crates/
│   ├── borno-core/                 # Unicode layout engine, state machine, phonetic transliteration
│   │   ├── src/bengali.rs          # Bengali Unicode normalization tables
│   │   ├── src/unicode.rs          # Borno Unicode layout mapping
│   │   ├── src/engine.rs           # Finite state machine and conjunct reordering
│   │   └── src/phonetic.rs         # Trie-based phonetic transliteration engine
│   ├── borno-cli/                  # Windows GUI floating toolbar and keyboard hook
│   │   ├── build.rs                # Windows PE resource and high-DPI manifest compiler
│   │   ├── icons/                  # Multi-resolution ICO and embedded graphics
│   │   ├── src/gui.rs              # Win32 double-buffered toolbar and system tray
│   │   ├── src/windows_hook.rs     # Low-level Windows keyboard hook implementation
│   │   └── src/main.rs             # CLI dispatcher and REPL
│   └── borno-installer/            # Native Win32 Setup Wizard executable
│       ├── build.rs                # Installer PE resource compiler
│       └── src/main.rs             # Multi-step wizard UI and payload extractor
└── scripts/
    ├── package-release.ps1         # Automated release packaging and hashing script
    ├── Install-BornoFree.cmd       # Command-line installation utility
    └── Uninstall-BornoFree.cmd     # System uninstallation script
```

---

## Building from Source

### Prerequisites
* Windows 10 or Windows 11 (64-bit)
* Rust toolchain (version 1.75 or newer)

### Compilation Commands

1. Clone the repository:
   ```bash
   git clone https://github.com/maruf7705/BornoFree.git
   cd BornoFree
   ```

2. Run automated test suite:
   ```bash
   cargo test --workspace
   ```

3. Compile optimized release binaries:
   ```bash
   cargo build --release --workspace
   ```

4. Generate distribution packages and setup installer:
   ```powershell
   powershell -ExecutionPolicy Bypass -File scripts/package-release.ps1 -Version "1.0.1"
   ```

---

## Command Line Interface

BornoFree includes integrated command-line utilities for phonetic transcription and interactive typing:

```bash
# Phonetic transliteration
BornoFree.exe phonetic "amar shonar bangla"

# Interactive terminal REPL
BornoFree.exe repl --mode borno-unicode
```

---

## License

BornoFree is licensed under the MIT License.

Copyright (c) 2026 Maruf Islam
