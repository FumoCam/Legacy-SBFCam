:: MAKE SURE TO RUN THE LINE BELOW IN A POWERSHELL WINDOW IF THIS IS THE FIRST TIME
:: Set-ExecutionPolicy RemoteSigned
@echo off
set OBS_DIR="%PROGRAMFILES%"\obs-studio\bin\64bit\
set SBFCAM_DIR=%USERPROFILE%\Desktop\SBFCam

:: Start OBS
cd /d %OBS_DIR%
start obs64.exe --disable-updater --startstreaming

:: Start CensorClient
cd /d %SBFCAM_DIR%\censor\
start poetry run python main.py
TIMEOUT /T 5

:: Start main program
cd /d %SBFCAM_DIR%\resources\
start powershell -file "./start_main.ps1"
