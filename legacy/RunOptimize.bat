@echo off
setlocal EnableExtensions

:: ============================================================
::  RunOptimize.bat -- Chay Optimize_Win11_For_Dev.ps1
::  Nhap doi: Nhap dup file nay la chay, khong can lam gi them
::  Yeu cau  : Windows 11, chay voi quyen Administrator
:: ============================================================

:: -- Kiem tra quyen Administrator --
net session >nul 2>&1
if %errorlevel% neq 0 (
    echo.
    echo  [!] Dang yeu cau quyen Administrator...
    echo.
    :: Tu dong tu nang cap quyen bang PowerShell UAC prompt
    powershell -NoProfile -Command ^
        "Start-Process -FilePath 'cmd.exe' -ArgumentList '/c \"%~f0\"' -Verb RunAs"
    exit /b
)

:: -- Xac dinh duong dan script cung thu muc voi file .bat --
set "SCRIPT_DIR=%~dp0"
set "PS1_FILE=%SCRIPT_DIR%Optimize_Win11_For_Dev.ps1"

:: -- Kiem tra file .ps1 ton tai khong --
if not exist "%PS1_FILE%" (
    echo.
    echo  [ERROR] Khong tim thay: %PS1_FILE%
    echo  Hay dam bao Optimize_Win11_For_Dev.ps1 cung thu muc voi RunOptimize.bat
    echo.
    pause
    exit /b 1
)

:: -- Thong bao bat dau --
echo.
echo  ================================================================
echo   WINDOWS 11 PRO -- DEV OPTIMIZER v3.0
echo   Dang chuan bi chay: Optimize_Win11_For_Dev.ps1
echo  ================================================================
echo.

:: -- Chay PowerShell script --
PowerShell -NoProfile -ExecutionPolicy Bypass -File "%PS1_FILE%"

:: -- Giu cua so mo neu PowerShell dong luon (truong hop khong restart) --
if %errorlevel% neq 0 (
    echo.
    echo  [WARN] Script ket thuc voi ma loi: %errorlevel%
    pause
)

endlocal
exit /b 0
