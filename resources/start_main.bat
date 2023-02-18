:: For manually starting after updates or something

:: MAKE SURE TO RUN THE LINE BELOW IN A POWERSHELL WINDOW IF THIS IS THE FIRST TIME
:: Set-ExecutionPolicy RemoteSigned
@echo off
set SBFCAM_DIR=%USERPROFILE%\Desktop\SBFCam
cd /d %SBFCAM_DIR%\resources\
start powershell -file "./start_main.ps1"
