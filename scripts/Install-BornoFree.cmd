@echo off
setlocal enabledelayedexpansion
title BornoFree v1.0.0 Installer

echo ======================================================================
echo             BornoFree (বর্ণফ্রি) v1.0.0 - Windows Installer
echo       Modern, Blazing-Fast Bengali Typing Engine for Windows 10/11
echo ======================================================================
echo.

:: Ensure BornoFree.exe exists in the current folder
if not exist "%~dp0BornoFree.exe" (
    echo [ERROR] BornoFree.exe was not found in this folder!
    echo Please make sure you have extracted all files from the zip.
    echo.
    pause
    exit /b 1
)

echo [1/5] Stopping any running BornoFree processes...
powershell -NoProfile -Command "Stop-Process -Name 'BornoFree' -Force -ErrorAction SilentlyContinue" >nul 2>&1

:: Target directory in user local application data
set "INSTALL_DIR=%LOCALAPPDATA%\Programs\BornoFree"
echo [2/5] Installing files to: %INSTALL_DIR%
if not exist "%INSTALL_DIR%" mkdir "%INSTALL_DIR%" >nul 2>&1

copy /Y "%~dp0BornoFree.exe" "%INSTALL_DIR%\BornoFree.exe" >nul
if exist "%~dp0README.txt" copy /Y "%~dp0README.txt" "%INSTALL_DIR%\README.txt" >nul
if exist "%~dp0LICENSE.txt" copy /Y "%~dp0LICENSE.txt" "%INSTALL_DIR%\LICENSE.txt" >nul
if exist "%~dp0Uninstall-BornoFree.cmd" copy /Y "%~dp0Uninstall-BornoFree.cmd" "%INSTALL_DIR%\Uninstall-BornoFree.cmd" >nul

echo [3/5] Creating Desktop and Start Menu shortcuts...
powershell -NoProfile -ExecutionPolicy Bypass -Command ^
  "$ws = New-Object -ComObject WScript.Shell; " ^
  "$desktop = [Environment]::GetFolderPath('Desktop'); " ^
  "$sc1 = $ws.CreateShortcut(\"$desktop\BornoFree.lnk\"); " ^
  "$sc1.TargetPath = '%INSTALL_DIR%\BornoFree.exe'; " ^
  "$sc1.WorkingDirectory = '%INSTALL_DIR%'; " ^
  "$sc1.IconLocation = '%INSTALL_DIR%\BornoFree.exe,0'; " ^
  "$sc1.Description = 'BornoFree - Modern Bengali Typing Engine'; " ^
  "$sc1.Save(); " ^
  "$startMenu = [Environment]::GetFolderPath('StartMenu') + '\Programs'; " ^
  "$sc2 = $ws.CreateShortcut(\"$startMenu\BornoFree.lnk\"); " ^
  "$sc2.TargetPath = '%INSTALL_DIR%\BornoFree.exe'; " ^
  "$sc2.WorkingDirectory = '%INSTALL_DIR%'; " ^
  "$sc2.IconLocation = '%INSTALL_DIR%\BornoFree.exe,0'; " ^
  "$sc2.Description = 'BornoFree - Modern Bengali Typing Engine'; " ^
  "$sc2.Save();"

echo [4/5] Registering BornoFree in Windows Add/Remove Programs...
powershell -NoProfile -ExecutionPolicy Bypass -Command ^
  "$regPath = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\BornoFree'; " ^
  "if (-not (Test-Path $regPath)) { New-Item -Path $regPath -Force | Out-Null }; " ^
  "Set-ItemProperty -Path $regPath -Name 'DisplayName' -Value 'BornoFree (বর্ণফ্রি)'; " ^
  "Set-ItemProperty -Path $regPath -Name 'DisplayVersion' -Value '1.0.0'; " ^
  "Set-ItemProperty -Path $regPath -Name 'Publisher' -Value 'BornoFree Open Source Project'; " ^
  "Set-ItemProperty -Path $regPath -Name 'DisplayIcon' -Value '%INSTALL_DIR%\BornoFree.exe'; " ^
  "Set-ItemProperty -Path $regPath -Name 'UninstallString' -Value '\"%INSTALL_DIR%\Uninstall-BornoFree.cmd\"'; " ^
  "Set-ItemProperty -Path $regPath -Name 'InstallLocation' -Value '%INSTALL_DIR%';"

echo.
echo ======================================================================
echo [SUCCESS] BornoFree v1.0.0 has been installed successfully!
echo   - Desktop Shortcut created
echo   - Start Menu Shortcut created
echo   - High-DPI 3D Blue Book icon applied
echo ======================================================================
echo.

set /p AUTOSTART="Do you want BornoFree to start automatically with Windows? (Y/N) [Default: Y]: "
if /i "%AUTOSTART%"=="" set AUTOSTART=Y
if /i "%AUTOSTART%"=="Y" (
    powershell -NoProfile -Command "Set-ItemProperty -Path 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run' -Name 'BornoFree' -Value '\"%INSTALL_DIR%\BornoFree.exe\"'"
    echo [OK] Auto-start on Windows boot enabled!
)

echo.
set /p LAUNCH="Launch BornoFree now? (Y/N) [Default: Y]: "
if /i "%LAUNCH%"=="" set LAUNCH=Y
if /i "%LAUNCH%"=="Y" (
    start "" "%INSTALL_DIR%\BornoFree.exe"
    echo [OK] BornoFree is now running! Enjoy typing in Bengali.
)

echo.
echo Press any key to exit installer.
pause >nul
exit /b 0
