:: MAKE SURE TO RUN THE LINE BELOW IN A POWERSHELL WINDOW IF THIS IS THE FIRST TIME
:: Set-ExecutionPolicy RemoteSigned
@echo off
cd "%PROGRAMFILES%\obs-studio\bin\64bit\"
start obs64.exe --minimize-to-tray --disable-updater --startstreaming
cd %USERPROFILE%\Desktop\SBFCam\resources
TIMEOUT /T 5
start powershell -file "./main.ps1"