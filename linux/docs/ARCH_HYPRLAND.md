# BornoFree on Arch Linux + Hyprland

BornoFree uses an Fcitx5 input-method addon on Linux. It has no floating
toolbar or window. The only visible state is Fcitx5's status icon in Waybar's
`tray` module: the BornoFree input method uses the `অ` label, and the US
keyboard shows its normal English indicator.

## Install

Install build and runtime requirements, then build the local Arch package:

```bash
sudo pacman -S --needed base-devel cmake ninja rust fcitx5 fcitx5-gtk fcitx5-qt waybar python
git clone https://github.com/maruf7705/BornoFree.git
cd BornoFree
makepkg -si
borno-hyprland-setup
```

`borno-hyprland-setup` creates backups in
`~/.local/state/borno-free/`, adds the Fcitx5 environment/startup settings and
Hyprland key binds to `~/.config/hypr/hyprland.conf`, adds BornoFree and
`keyboard-us` to the primary Fcitx5 profile group without removing its existing
input methods, and adds a `tray` module to a detected Waybar config. Log out
and back in once after setup.

To use another Hyprland config path, pass it explicitly:

```bash
borno-hyprland-setup --config ~/.config/hypr/hyprland.conf \
  --waybar-config ~/.config/waybar/config.jsonc
```

The setup helper targets Hyprland's `.conf` syntax. For a Lua configuration,
add the same environment values, `exec-once`, and bindings manually.

## Shortcuts

| Shortcut | Result |
| --- | --- |
| `F12` | Toggle English and Bengali |
| `Ctrl+Alt+V` | Select Borno Unicode (`অ`) |
| `Ctrl+Alt+P` | Select Avro Phonetic (`ফো`) |
| `Ctrl+Alt+E` | Select English (`EN`) |

The controller defaults to `keyboard-us` for English. Override it when your
Fcitx keyboard entry has another name:

```bash
BORNOFREE_ENGLISH_IM=keyboard-gb borno-fcitxctl english
```

## Verify and remove

Check the selected mode with:

```bash
borno-fcitxctl status
fcitx5-diagnose
```

In a GTK, Qt, browser, or XWayland text field, select Borno Unicode and type:

```text
g f m d              -> আমি
Shift+m d j g k      -> শক্তি
x k c r z f k        -> ওতপ্রোত
```

To restore the exact configs saved before setup:

```bash
borno-hyprland-setup --remove
```

The native Fcitx5 addon is installed at `/usr/lib/fcitx5/borno.so`; descriptor
files are installed under `/usr/share/fcitx5/`.
