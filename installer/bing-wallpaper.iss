#ifndef MyAppVersion
#define MyAppVersion "0.0.0"
#endif

[Setup]
AppId={{B7C0D7A5-6B73-4D6F-A1A0-9B7C9F6F1C32}
AppName=Bing Wallpaper
AppVersion={#MyAppVersion}
AppPublisher=wwb
AppPublisherURL=https://github.com/wwbgo/bing-wallpaper
DefaultDirName={localappdata}\Programs\Bing Wallpaper
DefaultGroupName=Bing Wallpaper
OutputDir=..\dist
OutputBaseFilename=BingWallpaperSetup-{#MyAppVersion}
Compression=lzma2
SolidCompression=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
PrivilegesRequired=lowest
WizardStyle=modern
UninstallDisplayIcon={app}\bing-wallpaper.exe

[Files]
Source: "..\target\release\bing-wallpaper.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\target\release\bing-wallpaper-worker.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\README.md"; DestDir: "{app}"; Flags: ignoreversion

[Run]
Filename: "{app}\bing-wallpaper.exe"; Parameters: "setup --time 09:30 --resolution auto --style fill"; Description: "立即更新并配置每日必应壁纸"; Flags: runhidden waituntilterminated

[UninstallRun]
Filename: "{app}\bing-wallpaper.exe"; Parameters: "uninstall"; Flags: runhidden
