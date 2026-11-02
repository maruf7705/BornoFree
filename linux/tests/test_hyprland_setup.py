"""Black-box tests for the user-scoped Hyprland installer."""

from __future__ import annotations

import hashlib
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SETUP = ROOT / "linux" / "bin" / "borno-hyprland-setup"


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


class HyprlandSetupTests(unittest.TestCase):
    def run_setup(self, env: dict[str, str], *args: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [sys.executable, str(SETUP), *args],
            cwd=ROOT,
            env=env,
            text=True,
            capture_output=True,
            check=False,
        )

    def fixture(self, profile: bool) -> tuple[tempfile.TemporaryDirectory[str], dict[str, str], Path, Path, Path]:
        temporary = tempfile.TemporaryDirectory()
        root = Path(temporary.name)
        config = root / "config"
        hypr = config / "hypr" / "hyprland.conf"
        waybar = config / "waybar" / "config.jsonc"
        fcitx_profile = config / "fcitx5" / "profile"
        hypr.parent.mkdir(parents=True)
        waybar.parent.mkdir(parents=True)
        hypr.write_text("monitor=,preferred,auto,1\n", encoding="utf-8")
        waybar.write_text('// keep comments\n{"modules-right": ["clock"]}\n', encoding="utf-8")
        if profile:
            fcitx_profile.parent.mkdir(parents=True)
            fcitx_profile.write_text(
                "[Groups/0]\nName=Default\n\n[Groups/0/Items/0]\nName=keyboard-bd\nLayout=bd\n",
                encoding="utf-8",
            )
        env = os.environ.copy()
        env["XDG_CONFIG_HOME"] = str(config)
        env["XDG_STATE_HOME"] = str(root / "state")
        return temporary, env, hypr, waybar, fcitx_profile

    def test_setup_is_idempotent_and_remove_restores_existing_files(self) -> None:
        temporary, env, hypr, waybar, profile = self.fixture(profile=True)
        self.addCleanup(temporary.cleanup)
        before = {path: digest(path) for path in (hypr, waybar, profile)}
        first = self.run_setup(env, "--config", str(hypr), "--waybar-config", str(waybar))
        self.assertEqual(first.returncode, 0, first.stderr)
        self.assertIn("updated", first.stdout)
        self.assertIn("borno-fcitxctl toggle", hypr.read_text(encoding="utf-8"))
        profile_text = profile.read_text(encoding="utf-8")
        self.assertIn("Name=borno", profile_text)
        self.assertIn("Name=keyboard-us", profile_text)
        self.assertIn("Name=keyboard-bd", profile_text)
        self.assertIn('"tray"', waybar.read_text(encoding="utf-8"))
        second = self.run_setup(env, "--config", str(hypr), "--waybar-config", str(waybar))
        self.assertEqual(second.returncode, 0, second.stderr)
        self.assertIn("already configured", second.stdout)
        removed = self.run_setup(env, "--remove")
        self.assertEqual(removed.returncode, 0, removed.stderr)
        self.assertEqual(before, {path: digest(path) for path in (hypr, waybar, profile)})

    def test_remove_deletes_profile_created_by_setup(self) -> None:
        temporary, env, hypr, waybar, profile = self.fixture(profile=False)
        self.addCleanup(temporary.cleanup)
        created = self.run_setup(env, "--config", str(hypr), "--waybar-config", str(waybar))
        self.assertEqual(created.returncode, 0, created.stderr)
        self.assertTrue(profile.exists())
        removed = self.run_setup(env, "--remove")
        self.assertEqual(removed.returncode, 0, removed.stderr)
        self.assertFalse(profile.exists())


if __name__ == "__main__":
    unittest.main()