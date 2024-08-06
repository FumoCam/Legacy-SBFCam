@echo off
set SBFCAM_DIR=%USERPROFILE%\Desktop\SBFCam

:: Public Repos
:: SBFCam (This repo, https://github.com/FumoCam/Legacy-SBFCam)
echo.
echo Updating SBFCam...
cd /d %SBFCAM_DIR%
git fetch --all
git reset --hard origin/main
cd /d %SBFCAM_DIR%\python\
poetry install
pause
:: Censor-Client (https://github.com/FumoCam/Whitelist-Censor-Client)
echo.
echo Updating Censor-Client...
cd /d %SBFCAM_DIR%\censor\
git fetch --all
git reset --hard origin/main
poetry install
pause
:: HUD (https://github.com/FumoCam/HUD)
echo.
echo Updating HUD...
cd /d %SBFCAM_DIR%\hud\
git fetch --all
git reset --hard origin/main
pause

:: Private Repos
:: SBFCam Private Resources (https://github.com/FumoCam/Legacy-SBFCam-PrivateResources)
echo.
echo Updating Private Resources...
cd /d %SBFCAM_DIR%\resources\private\
git fetch --all
git reset --hard origin/main
pause
:: HUD Private Assets (https://github.com/FumoCam/HUD-PrivateAssets)
echo.
echo Updating HUD Private Assets...
cd /d %SBFCAM_DIR%\hud\private_assets\
git fetch --all
git reset --hard origin/main
pause
