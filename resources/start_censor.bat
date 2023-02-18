:: For manually starting after updates or something
@echo off
set SBFCAM_DIR=%USERPROFILE%\Desktop\SBFCam
cd /d %SBFCAM_DIR%\censor\
start poetry run python main.py
