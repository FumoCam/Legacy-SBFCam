# MAKE SURE TO RUN THE LINE BELOW IN A POWERSHELL WINDOW IF THIS IS THE FIRST TIME
# Set-ExecutionPolicy RemoteSigned

#===RUN AS ADMIN===
if (!([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    Start-Process PowerShell -Verb RunAs "-NoProfile -ExecutionPolicy Bypass -Command `"cd '$pwd'; & '$PSCommandPath';`"";
    exit;
}
#===END RUN-AS-ADMIN===

#Script
# Write-Host -NoNewLine "Press any key to continue...";
# $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown");
# poetry run python main.py
# echo "Waiting 5 seconds..."
# Start-Sleep -Seconds 5
# [System.Threading.Thread]::Sleep(5000)

cd ..\rust
cargo run --release
