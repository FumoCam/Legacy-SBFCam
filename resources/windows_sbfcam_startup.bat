:: MAKE SURE TO RUN THE LINE BELOW IN A POWERSHELL WINDOW IF THIS IS THE FIRST TIME
:: Set-ExecutionPolicy RemoteSigned
@echo off

cd "%PROGRAMFILES%\obs-studio\bin\64bit\"
start obs64.exe --minimize-to-tray --disable-updater --startstreaming
TIMEOUT /T 5

cd %USERPROFILE%\Desktop\CensorClient
start poetry run python main.py
TIMEOUT /T 5

cd %USERPROFILE%\Desktop\SBFCam\resources
start powershell -file "./main.ps1"