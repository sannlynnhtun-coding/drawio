#ifndef SourceExe
  #error Pass /DSourceExe=<path to the bundled draw.io executable>.
#endif

[Setup]
AppId={{70AA4A28-3374-48EB-9ADC-A14A52070225}
AppName=Drawio
AppVersion=1.0
VersionInfoVersion=1.0.0.0
DefaultDirName={localappdata}\Programs\DrawioDesktop
DefaultGroupName=Drawio
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0
OutputBaseFilename=Drawio-Setup-x64
Compression=lzma2/fast
SolidCompression=yes
SetupIconFile=assets\Drawio.ico
UninstallDisplayIcon={app}\Drawio.exe

[Files]
Source: "{#SourceExe}"; DestDir: "{app}"; DestName: "Drawio.exe"; Flags: ignoreversion
Source: "..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{userprograms}\Drawio"; Filename: "{app}\Drawio.exe"

[Run]
Filename: "{app}\Drawio.exe"; Description: "Open Drawio"; Flags: nowait postinstall skipifsilent


[InstallDelete]
Type: files; Name: "{userprograms}\Draw.io Desktop.lnk"
