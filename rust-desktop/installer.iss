#ifndef SourceExe
  #error Pass /DSourceExe=<path to the bundled draw.io executable>.
#endif

[Setup]
AppId={{70AA4A28-3374-48EB-9ADC-A14A52070225}
AppName=Drawio
AppVersion=1.0.1
VersionInfoVersion=1.0.1.0
DefaultDirName={localappdata}\Programs\DrawioDesktop
DefaultGroupName=Drawio
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0
ChangesAssociations=yes
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

[Registry]
Root: HKCU; Subkey: "Software\Classes\.drawio\OpenWithProgids"; ValueType: string; ValueName: "Drawio.Diagram"; ValueData: ""; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\Drawio.Diagram"; ValueType: string; ValueData: "Drawio Diagram"; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\Drawio.Diagram\DefaultIcon"; ValueType: string; ValueData: """{app}\Drawio.exe"",0"
Root: HKCU; Subkey: "Software\Classes\Drawio.Diagram\shell\open\command"; ValueType: string; ValueData: """{app}\Drawio.exe"" ""%1"""
Root: HKCU; Subkey: "Software\Classes\Applications\Drawio.exe"; ValueType: string; ValueName: "FriendlyAppName"; ValueData: "Drawio"; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\Applications\Drawio.exe\SupportedTypes"; ValueType: string; ValueName: ".drawio"; ValueData: ""
Root: HKCU; Subkey: "Software\Classes\Applications\Drawio.exe\shell\open\command"; ValueType: string; ValueData: """{app}\Drawio.exe"" ""%1"""

[Run]
Filename: "{app}\Drawio.exe"; Description: "Open Drawio"; Flags: nowait postinstall skipifsilent


[InstallDelete]
Type: files; Name: "{userprograms}\Draw.io Desktop.lnk"
