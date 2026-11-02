Current: Arch + Hyprland / BornoFree uses Fcitx5 as its input method. It has no floating toolbar: Fcitx sends Bangla text into the focused app, and the mode indicator appears in Waybar’s tray. / Next: On Arch, install and set it up:

```bash
sudo pacman -Syu --needed git base-devel cmake ninja rust fcitx5 fcitx5-gtk fcitx5-qt waybar python
git clone https://github.com/maruf7705/BornoFree.git
cd BornoFree
makepkg -si
borno-hyprland-setup
```

If you already cloned the repo, run `git pull` in its folder instead of cloning again. Setup backs up your Hyprland and Fcitx settings, adds the Fcitx startup and shortcuts, and adds Waybar’s `tray` module. Make sure you already have a Waybar config; if it’s somewhere non-standard, pass it with `--waybar-config <path>`.

Log out and back into Hyprland after setup. Then:

- `F12` toggles English ↔ Bangla
- `Ctrl+Alt+V` selects Bangla
- `Ctrl+Alt+E` selects English
- `borno-fcitxctl status` reports the selected mode

Try `g f m d` → **আমি**, `Shift+m d j g k` → **শক্তি**, and `x k c r z f k` → **ওতপ্রোত**. You can check Fcitx with `fcitx5-diagnose`. To undo the Hyprland/Fcitx config changes, run `borno-hyprland-setup --remove`.

ArchWiki notes that GTK/Qt modules help with toolkit integration, while native Wayland text-input is generally preferred where supported; XWayland apps may need the modules. [ArchWiki: Fcitx5](https://wiki.archlinux.org/title/Fcitx5)