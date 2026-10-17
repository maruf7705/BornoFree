@echo off
setlocal enabledelayedexpansion
title BornoFree Uninstaller

echo ======================================================================
echo             BornoFree (বর্ণফ্রি) - Windows Uninstaller
echo ======================================================================
echo.

set /p CONFIRM="Are you sure you want to completely uninstall BornoFree? (Y/N): "
if /i not "%CONFIRM%"=="Y" (
    echo Uninstallation cancelled.
    pause
    exit /b 0
)

echo.
echo [1/4] Closing BornoFree process...
powershell -NoProfile -Command "Stop-Process -Name 'BornoFree' -Force -ErrorAction SilentlyContinue" >nul 2>&1

echo [2/4] Removing Desktop and Start Menu shortcuts...
powershell -NoProfile -ExecutionPolicy Bypass -Command ^
  "$desktop = [Environment]::GetFolderPath('Desktop'); " ^
  "Remove-Item -Path \"$desktop\BornoFree.lnk\" -Force -ErrorAction SilentlyContinue; " ^
  "$startMenu = [Environment]::GetFolderPath('StartMenu') + '\Programs'; " ^
  "Remove-Item -Path \"$startMenu\BornoFree.lnk\" -Force -ErrorAction SilentlyContinue;"

echo [3/4] Cleaning Registry settings...
powershell -NoProfile -ExecutionPolicy Bypass -Command ^
  "Remove-Item -Path 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\BornoFree' -Recurse -Force -ErrorAction SilentlyContinue; " ^
  "Remove-ItemProperty -Path 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run' -Name 'BornoFree' -Force -ErrorAction SilentlyContinue;"

echo [4/4] Removing installed program files...
set "INSTALL_DIR=%LOCALAPPDATA%\Programs\BornoFree"
if exist "%INSTALL_DIR%" (
    rmdir /S /Q "%INSTALL_DIR%" >nul 2>&1
)

echo.
echo ======================================================================
echo [SUCCESS] BornoFree has been completely uninstalled from your system.
echo ======================================================================
echo.
pause
exit /b 0
