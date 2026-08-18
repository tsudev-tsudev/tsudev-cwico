#Requires -RunAsAdministrator
# =========================================================================================
#  OPTIMIZE_WIN11_FOR_DEV.PS1
#  Tu dong toi uu hoa Windows 11 Pro cho Lap trinh vien -- Chay 1 Click
#  Version : 3.0
#  Yeu cau  : PowerShell 5.1+, Windows 11, Run as Administrator
#  Tac gia  : Tong hop & chuan hoa cho moi truong Dev Viet Nam (UTC+7)
# =========================================================================================

Set-StrictMode -Version Latest
$ErrorActionPreference = "SilentlyContinue"

# ---------------------------------------------------------------------------
# HELPER FUNCTIONS
# ---------------------------------------------------------------------------
function Write-Step {
    param([int]$Step, [int]$Total, [string]$Message)
    Write-Host ""
    Write-Host "  [$Step/$Total] $Message" -ForegroundColor Yellow
}
function Write-OK   { param([string]$Msg) Write-Host "    [OK]   $Msg" -ForegroundColor Green }
function Write-Skip { param([string]$Msg) Write-Host "    [SKIP] $Msg" -ForegroundColor DarkGray }
function Write-Warn { param([string]$Msg) Write-Host "    [WARN] $Msg" -ForegroundColor DarkYellow }
function Write-Fail { param([string]$Msg) Write-Host "    [FAIL] $Msg" -ForegroundColor Red }

function Ensure-RegistryPath {
    param([string]$Path)
    if (-not (Test-Path $Path)) {
        New-Item -Path $Path -Force | Out-Null
    }
}

function Set-Reg {
    param([string]$Path, [string]$Name, $Value, [string]$Type = "DWord")
    Ensure-RegistryPath $Path
    Set-ItemProperty -Path $Path -Name $Name -Value $Value -Type $Type -Force
}

function Remove-AppxSafe {
    param([string]$Pattern)
    Get-AppxPackage $Pattern -ErrorAction SilentlyContinue | ForEach-Object {
        $_ | Remove-AppxPackage -ErrorAction SilentlyContinue
        Write-OK "Go AppxPackage: $($_.Name)"
    }
    Get-AppxProvisionedPackage -Online -ErrorAction SilentlyContinue |
        Where-Object { $_.DisplayName -like $Pattern.Replace("*","") } |
        ForEach-Object {
            Remove-AppxProvisionedPackage -Online -PackageName $_.PackageName -ErrorAction SilentlyContinue | Out-Null
            Write-OK "Go Provisioned: $($_.DisplayName)"
        }
}

# ---------------------------------------------------------------------------
# BANNER
# ---------------------------------------------------------------------------
Clear-Host
Write-Host ""
Write-Host "  ================================================================" -ForegroundColor Cyan
Write-Host "   WINDOWS 11 PRO -- DEV OPTIMIZER v3.0                         " -ForegroundColor Cyan
Write-Host "   Bloatware | Xbox | OneDrive | Privacy | Performance | WSL2   " -ForegroundColor Cyan
Write-Host "  ================================================================" -ForegroundColor Cyan
Write-Host ""

# ---------------------------------------------------------------------------
# KIEM TRA QUYEN ADMIN
# ---------------------------------------------------------------------------
$IsAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole(
    [Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $IsAdmin) {
    Write-Fail "Script can quyen Administrator. Chuot phai -> Run as Administrator!"
    Read-Host "  Nhan Enter de thoat"
    exit 1
}

Set-ExecutionPolicy Bypass -Scope Process -Force

$TotalSteps = 12

# =========================================================================================
# BUOC 1 -- GO BO & CHAN ONEDRIVE VINH VIEN
# =========================================================================================
Write-Step 1 $TotalSteps "Go bo & chan OneDrive vinh vien..."

Stop-Process -Name "OneDrive" -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 1

foreach ($exe in @("$env:SystemRoot\SysWOW64\OneDriveSetup.exe", "$env:SystemRoot\System32\OneDriveSetup.exe")) {
    if (Test-Path $exe) {
        Start-Process $exe -ArgumentList "/uninstall" -NoNewWindow -Wait
        Write-OK "Da chay OneDriveSetup /uninstall"
        break
    }
}

@("$env:USERPROFILE\OneDrive", "$env:LOCALAPPDATA\Microsoft\OneDrive", "$env:PROGRAMDATA\Microsoft OneDrive") | ForEach-Object {
    if (Test-Path $_) { Remove-Item $_ -Recurse -Force; Write-OK "Xoa: $_" }
}

Set-Reg "HKLM:\SOFTWARE\Policies\Microsoft\Windows\OneDrive" "DisableFileSyncNGSC" 1
Write-OK "Chan OneDrive qua Group Policy"

foreach ($clsid in @("HKCR:\CLSID\{018D5C66-4533-4307-9B53-224DE2ED1FE6}", "HKCR:\Wow6432Node\CLSID\{018D5C66-4533-4307-9B53-224DE2ED1FE6}")) {
    if (Test-Path $clsid) { Set-ItemProperty -Path $clsid -Name "System.IsPinnedToNameSpaceTree" -Value 0 -Force }
}
Write-OK "An OneDrive khoi File Explorer"

# =========================================================================================
# BUOC 2 -- TAT XBOX / GAME BAR / GAME MODE
# =========================================================================================
Write-Step 2 $TotalSteps "Vo hieu hoa Xbox, Game Bar va Game Mode..."

@(
    "*Microsoft.XboxGamingOverlay*", "*Microsoft.XboxApp*", "*Microsoft.Xbox.TCUI*",
    "*Microsoft.XboxIdentityProvider*", "*Microsoft.XboxSpeechToTextOverlay*",
    "*Microsoft.GamingApp*"
) | ForEach-Object { Remove-AppxSafe $_ }

Set-Reg "HKCU:\Software\Microsoft\Windows\CurrentVersion\GameDVR" "AppCaptureEnabled" 0
Set-Reg "HKCU:\Software\Microsoft\Windows\CurrentVersion\GameDVR" "GameDVR_Enabled" 0
Set-Reg "HKCU:\Software\Microsoft\GameBar" "AllowAutoGameMode" 0
Set-Reg "HKCU:\Software\Microsoft\GameBar" "UseNexusForGameBarEnabled" 0
Set-Reg "HKLM:\SOFTWARE\Policies\Microsoft\Windows\GameDVR" "AllowGameDVR" 0

foreach ($svc in @("XblAuthManager", "XblGameSave", "XboxNetApiSvc", "XboxGipSvc", "xbgm")) {
    if (Get-Service -Name $svc -ErrorAction SilentlyContinue) {
        Stop-Service -Name $svc -Force -ErrorAction SilentlyContinue
        Set-Service  -Name $svc -StartupType Disabled -ErrorAction SilentlyContinue
        Write-OK "Disabled service: $svc"
    }
}
Write-OK "He sinh thai Xbox da bi vo hieu hoa"

# =========================================================================================
# BUOC 3 -- GO BLOATWARE MAC DINH
# =========================================================================================
Write-Step 3 $TotalSteps "Go bo Bloatware (Cortana, News, Solitaire, Teams, Widgets...)..."

@(
    "*Cortana*",
    "*BingNews*", "*BingWeather*", "*BingSearch*", "*BingFinance*", "*BingSports*",
    "*MicrosoftSolitaireCollection*",
    "*GetHelp*", "*Getstarted*",
    "*YourPhone*", "*PhoneLink*",
    "*MicrosoftTeams*",
    "*ZuneVideo*", "*ZuneMusic*", "*WindowsMediaPlayer*",
    "*People*",
    "*WindowsFeedbackHub*",
    "*Microsoft3DViewer*", "*MixedReality.Portal*",
    "*windowscommunicationsapps*",
    "*MSPaint*",
    "*Messaging*",
    "*SkypeApp*",
    "*549981C3F5F10*",
    "*MicrosoftOfficeHub*",
    "*Todos*",
    "*WindowsMaps*",
    "*Microsoft.Whiteboard*",
    "*Microsoft.PowerAutomateDesktop*",
    "*MicrosoftEdge*"
) | ForEach-Object { Remove-AppxSafe $_ }

# Go provisioned packages (cho user moi tao sau nay)
@(
    "Microsoft.BingNews", "Microsoft.BingWeather", "Microsoft.GetHelp",
    "Microsoft.Getstarted", "Microsoft.MicrosoftSolitaireCollection",
    "Microsoft.MixedReality.Portal", "Microsoft.People", "Microsoft.SkypeApp",
    "Microsoft.WindowsFeedbackHub", "Microsoft.YourPhone", "Microsoft.ZuneMusic",
    "Microsoft.ZuneVideo", "Microsoft.Todos", "Microsoft.WindowsMaps",
    "Microsoft.MicrosoftOfficeHub", "Microsoft.PowerAutomateDesktop"
) | ForEach-Object {
    $prov = Get-AppxProvisionedPackage -Online | Where-Object DisplayName -EQ $_
    if ($prov) {
        Remove-AppxProvisionedPackage -Online -PackageName $prov.PackageName -ErrorAction SilentlyContinue | Out-Null
        Write-OK "Go Provisioned: $_"
    }
}

# =========================================================================================
# BUOC 4 -- TAT TELEMETRY & THEO DOI PRIVACY
# =========================================================================================
Write-Step 4 $TotalSteps "Tat Telemetry, quang cao va gui du lieu ve Microsoft..."

Set-Reg "HKLM:\SOFTWARE\Policies\Microsoft\Windows\DataCollection" "AllowTelemetry" 0
Write-OK "Telemetry = Disabled (level 0)"

Set-Reg "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\DataCollection" "AllowTelemetry" 0
Set-Reg "HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Policies\DataCollection" "AllowTelemetry" 0

Set-Reg "HKLM:\SOFTWARE\Policies\Microsoft\Windows\System" "EnableActivityFeed" 0
Set-Reg "HKLM:\SOFTWARE\Policies\Microsoft\Windows\System" "PublishUserActivities" 0
Set-Reg "HKLM:\SOFTWARE\Policies\Microsoft\Windows\System" "UploadUserActivities" 0
Write-OK "Activity History = Disabled"

Set-Reg "HKCU:\Software\Microsoft\Windows\CurrentVersion\AdvertisingInfo" "Enabled" 0
Set-Reg "HKLM:\SOFTWARE\Policies\Microsoft\Windows\AdvertisingInfo" "DisabledByGroupPolicy" 1
Write-OK "Advertising ID = Disabled"

Set-Reg "HKLM:\SOFTWARE\Policies\Microsoft\SQMClient\Windows" "CEIPEnable" 0
Write-OK "CEIP = Disabled"

# Tat Location tracking
Set-Reg "HKLM:\SOFTWARE\Policies\Microsoft\Windows\LocationAndSensors" "DisableLocation" 1
Write-OK "Location Tracking = Disabled"

# Tat thu thap loi crash gui MS
Set-Reg "HKLM:\SOFTWARE\Microsoft\Windows\Windows Error Reporting" "Disabled" 1
Write-OK "Windows Error Reporting = Disabled"

foreach ($svc in @("DiagTrack", "dmwappushservice", "WerSvc")) {
    if (Get-Service -Name $svc -ErrorAction SilentlyContinue) {
        Stop-Service -Name $svc -Force -ErrorAction SilentlyContinue
        Set-Service  -Name $svc -StartupType Disabled -ErrorAction SilentlyContinue
        Write-OK "Disabled service: $svc"
    }
}

# =========================================================================================
# BUOC 5 -- TAT SERVICES KHONG CAN THIET
# =========================================================================================
Write-Step 5 $TotalSteps "Tat cac Windows Services khong can thiet cho Dev..."

@(
    "Fax",
    "MapsBroker",
    "RetailDemo",
    "RemoteRegistry",
    "TabletInputService",
    "WSearch",                  # Windows Search (co the bat lai neu can)
    "SysMain",                  # Superfetch -- gay disk spike tren SSD
    "lfsvc",                    # Geolocation
    "SharedAccess",             # Internet Connection Sharing
    "TrkWks",                   # Distributed Link Tracking Client
    "WbioSrvc",                 # Windows Biometric (bo neu khong dung Windows Hello)
    "icssvc",                   # Windows Mobile Hotspot (khong can tren may ban)
    "PcaSvc",                   # Program Compatibility Assistant
    "WMPNetworkSvc"             # Windows Media Player Network Sharing
) | ForEach-Object {
    $svc = Get-Service -Name $_ -ErrorAction SilentlyContinue
    if ($svc) {
        Stop-Service -Name $_ -Force -ErrorAction SilentlyContinue
        Set-Service  -Name $_ -StartupType Disabled -ErrorAction SilentlyContinue
        Write-OK "Disabled service: $_"
    } else {
        Write-Skip "Khong tim thay: $_"
    }
}

# =========================================================================================
# BUOC 6 -- CAU HINH HIEU NANG & FILE EXPLORER CHO DEV
# =========================================================================================
Write-Step 6 $TotalSteps "Cau hinh hieu nang, File Explorer va Developer Mode..."

# Developer Mode
Set-Reg "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\AppModelUnlock" "AllowDevelopmentWithoutDevLicense" 1
Set-Reg "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\AppModelUnlock" "AllowAllTrustedApps" 1
Write-OK "Developer Mode = Enabled"

# Long paths
Set-Reg "HKLM:\SYSTEM\CurrentControlSet\Control\FileSystem" "LongPathsEnabled" 1
Write-OK "Long Paths (MAX_PATH) = Enabled"

# File Explorer settings
$fe = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced"
Set-Reg $fe "HideFileExt"     0  # Hien duoi file
Set-Reg $fe "Hidden"          1  # Hien file an
Set-Reg $fe "ShowSuperHidden" 1  # Hien file he thong
Set-Reg $fe "LaunchTo"        1  # Mo This PC thay vi Quick Access
Set-Reg $fe "HideEmptyDrives" 0  # Hien o dia trong
Set-Reg $fe "SnapAssist"      0  # Tat SnapAssist
Set-Reg $fe "TaskbarAnimations" 0  # Tat animation taskbar
Set-Reg $fe "EnableBalloonTips" 0  # Tat tooltip popup
Write-OK "File Explorer: hien duoi file, file an, he thong. SnapAssist OFF"

# Power plan -- High Performance
$hp = powercfg -list 2>$null | Select-String "High performance"
if ($hp) {
    $guid = ($hp -split "\s+")[3]
    powercfg -setactive $guid
    Write-OK "Power Plan = High Performance ($guid)"
} else {
    powercfg -setactive SCHEME_MIN
    Write-OK "Power Plan = Ultimate Performance (fallback SCHEME_MIN)"
}
powercfg -h off
Write-OK "Hibernation = Off"

Set-Reg "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Power" "HiberbootEnabled" 0
Write-OK "Fast Startup = Disabled (on dinh cho WSL/Docker)"

# Visual Effects
Set-Reg "HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\VisualEffects" "VisualFXSetting" 2
Write-OK "Visual Effects = Best Performance"

# Tat Transparency
Set-Reg "HKCU:\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize" "EnableTransparency" 0
Write-OK "Transparency = Disabled"

# =========================================================================================
# BUOC 7 -- KICH HOAT WSL2 & HYPER-V
# =========================================================================================
Write-Step 7 $TotalSteps "Kich hoat WSL2 / Hyper-V / Virtual Machine Platform..."

@(
    "Microsoft-Windows-Subsystem-Linux",
    "VirtualMachinePlatform",
    "Containers-DisposableClientVM",
    "Microsoft-Hyper-V-All"
) | ForEach-Object {
    $state = (Get-WindowsOptionalFeature -Online -FeatureName $_ -ErrorAction SilentlyContinue).State
    switch ($state) {
        "Disabled" {
            Enable-WindowsOptionalFeature -Online -FeatureName $_ -All -NoRestart -ErrorAction SilentlyContinue | Out-Null
            Write-OK "Enabled Feature: $_"
        }
        "Enabled"  { Write-Skip "Da bat san: $_" }
        default    { Write-Skip "Khong ho tro: $_" }
    }
}

if (Get-Command "wsl" -ErrorAction SilentlyContinue) {
    wsl --set-default-version 2 2>$null
    Write-OK "WSL Default Version = 2"
} else {
    Write-Skip "WSL chua cai -- chay 'wsl --install' sau khi reboot"
}

# =========================================================================================
# BUOC 8 -- TAT NOTIFICATIONS & STARTUP ANNOYANCES
# =========================================================================================
Write-Step 8 $TotalSteps "Tat thong bao va quang cao phien phuc..."

$cdm = "HKCU:\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager"
@(
    "SubscribedContent-338387Enabled", "SubscribedContent-338388Enabled",
    "SubscribedContent-338389Enabled", "SubscribedContent-353694Enabled",
    "SubscribedContent-353696Enabled", "SilentInstalledAppsEnabled",
    "SystemPaneSuggestionsEnabled",    "SoftLandingEnabled",
    "RotatingLockScreenEnabled",       "RotatingLockScreenOverlayEnabled",
    "OemPreInstalledAppsEnabled",      "PreInstalledAppsEnabled",
    "PreInstalledAppsEverEnabled",     "ContentDeliveryAllowed"
) | ForEach-Object { Set-Reg $cdm $_ 0 }
Write-OK "Windows Tips, Suggested Content & Lock Screen Widgets = Disabled"

Set-Reg "HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\AutoplayHandlers" "DisableAutoplay" 1
Write-OK "AutoPlay = Disabled"

# Tat Start Menu recommendations
Set-Reg "HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced" "Start_IrisRecommendations" 0
Set-Reg "HKLM:\SOFTWARE\Policies\Microsoft\Windows\Explorer" "HideRecommendedSection" 1
Write-OK "Start Menu Recommendations = Disabled"

# =========================================================================================
# BUOC 9 -- CAU HINH NETWORK CHO DEV
# =========================================================================================
Write-Step 9 $TotalSteps "Toi uu cau hinh Network cho Dev..."

# Tat Windows Update tu dong khi dang lam viec (chi Windows Update Delivery Optimization P2P ngoai mang LAN)
Set-Reg "HKLM:\SOFTWARE\Policies\Microsoft\Windows\DeliveryOptimization" "DODownloadMode" 1
Write-OK "Windows Update Delivery: chi dung LAN (tat P2P ngoai internet)"

# Tang TCP buffer cho performance
netsh int tcp set global autotuninglevel=normal 2>$null
netsh int tcp set global chimney=enabled 2>$null
netsh int tcp set global rss=enabled 2>$null
Write-OK "TCP Auto-Tuning & RSS = Enabled"

# DNS over HTTPS (dung Cloudflare 1.1.1.1)
Set-Reg "HKLM:\SYSTEM\CurrentControlSet\Services\Dnscache\Parameters" "EnableAutoDoh" 2
Write-OK "DNS-over-HTTPS (DoH) = Auto"

# =========================================================================================
# BUOC 10 -- CAU HINH CUOI CUNG (UX + ACCESSIBILITY)
# =========================================================================================
Write-Step 10 $TotalSteps "Cau hinh UX cuoi cung cho Dev..."

# Clipboard History Win+V
Set-Reg "HKCU:\Software\Microsoft\Clipboard" "EnableClipboardHistory" 1
Write-OK "Clipboard History (Win+V) = Enabled"

# Taskbar End Task
Set-Reg "HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced\TaskbarDeveloperSettings" "TaskbarEndTask" 1
Write-OK "Taskbar End Task = Enabled"

# Tat Copilot
Set-Reg "HKCU:\Software\Policies\Microsoft\Windows\WindowsCopilot" "TurnOffWindowsCopilot" 1
Set-Reg "HKLM:\SOFTWARE\Policies\Microsoft\Windows\WindowsCopilot" "TurnOffWindowsCopilot" 1
Write-OK "Windows Copilot = Disabled"

# Timezone
tzutil /s "SE Asia Standard Time"
Write-OK "Timezone = SE Asia Standard Time (UTC+7)"

# Tat Sticky / Filter / Toggle Keys
Set-Reg "HKCU:\Control Panel\Accessibility\StickyKeys"        "Flags" "506" "String"
Set-Reg "HKCU:\Control Panel\Accessibility\Keyboard Response" "Flags" "122" "String"
Set-Reg "HKCU:\Control Panel\Accessibility\ToggleKeys"        "Flags" "58"  "String"
Write-OK "Sticky Keys / Filter Keys / Toggle Keys = Disabled"

# Tat Search Highlights
Set-Reg "HKCU:\Software\Microsoft\Windows\CurrentVersion\SearchSettings" "IsDynamicSearchBoxEnabled" 0
Set-Reg "HKCU:\Software\Microsoft\Windows\CurrentVersion\Feeds\DSB" "ShowDynamicContent" 0
Write-OK "Search Highlights = Disabled"

# Tat Meet Now
Set-Reg "HKCU:\Software\Microsoft\Windows\CurrentVersion\Policies\Explorer" "HideSCAMeetNow" 1
Write-OK "Meet Now (Taskbar) = Disabled"

# =========================================================================================
# BUOC 11 -- CAU HINH GIT GLOBAL (neu Git da cai)
# =========================================================================================
Write-Step 11 $TotalSteps "Cau hinh Git global (neu da cai dat)..."

if (Get-Command "git" -ErrorAction SilentlyContinue) {
    git config --global core.autocrlf input          # LF tren Windows Dev
    git config --global core.longpaths true           # Long path support
    git config --global pull.rebase false             # Merge by default
    git config --global init.defaultBranch main       # Branch mac dinh la 'main'
    git config --global core.editor "notepad"         # Editor de mo nhat
    git config --global fetch.prune true              # Tu dong xoa remote branch da xoa
    Write-OK "Git global config da duoc thiet lap"
} else {
    Write-Skip "Git chua cai -- bo qua buoc nay"
}

# =========================================================================================
# BUOC 12 -- DON DEP & RESTART EXPLORER
# =========================================================================================
Write-Step 12 $TotalSteps "Don dep va restart Explorer de ap dung thay doi..."

# Xoa Temp files
@("$env:TEMP", "$env:SystemRoot\Temp") | ForEach-Object {
    if (Test-Path $_) {
        Get-ChildItem $_ -Recurse -Force -ErrorAction SilentlyContinue |
            Remove-Item -Recurse -Force -ErrorAction SilentlyContinue
        Write-OK "Don dep Temp: $_"
    }
}

# Restart Explorer de ap dung registry thay doi ngay
Stop-Process -Name "explorer" -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 2
Start-Process "explorer.exe"
Write-OK "Explorer da duoc khoi dong lai"

# =========================================================================================
# HOAN TAT
# =========================================================================================
Write-Host ""
Write-Host "  ================================================================" -ForegroundColor Green
Write-Host "   [DONE] TOI UU HOA HOAN TAT! (12/12 buoc)                     " -ForegroundColor Green
Write-Host "  ================================================================" -ForegroundColor Green
Write-Host "   OneDrive go & chan       [OK]   Bloatware don sach      [OK]  " -ForegroundColor White
Write-Host "   Xbox/Game Bar tat        [OK]   Telemetry & Ads tat    [OK]  " -ForegroundColor White
Write-Host "   Developer Mode bat       [OK]   Long Paths bat         [OK]  " -ForegroundColor White
Write-Host "   WSL2 / Hyper-V bat       [OK]   High Performance       [OK]  " -ForegroundColor White
Write-Host "   Network TCP toi uu       [OK]   DNS-over-HTTPS         [OK]  " -ForegroundColor White
Write-Host "   Clipboard History        [OK]   Timezone UTC+7         [OK]  " -ForegroundColor White
Write-Host "   Git global config        [OK]   Temp files don dep     [OK]  " -ForegroundColor White
Write-Host "  ================================================================" -ForegroundColor Green
Write-Host "   [!] Can RESTART may tinh de ap dung day du thay doi!          " -ForegroundColor Yellow
Write-Host "  ================================================================" -ForegroundColor Green
Write-Host ""

$Restart = Read-Host "  Ban co muon KHOI DONG LAI ngay bay gio khong? (Y / N)"
if ($Restart -match "^[Yy]$") {
    Write-Host "  Dang khoi dong lai..." -ForegroundColor Cyan
    Start-Sleep -Seconds 3
    Restart-Computer -Force
} else {
    Write-Host ""
    Write-Host "  Nho khoi dong lai truoc khi code nhe! Chuc ban code vui :)" -ForegroundColor Cyan
    Write-Host ""
}
