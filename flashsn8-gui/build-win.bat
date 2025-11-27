@echo off
setlocal enabledelayedexpansion

REM Get version as the argument
set VERSION=%1
if not defined VERSION set VERSION=0.0.1

REM Nuitka compile
python -m nuitka --standalone --enable-plugin=pyside6 --user-package-configuration-file=config.yaml --windows-console-mode=disable --output-dir=win flashsn8-gui.py

REM Remove old directory if it exists
cd win
if exist flashsn8-gui-win-%VERSION% rmdir /s /q flashsn8-gui-win-%VERSION%
rename flashsn8-gui.dist flashsn8-gui-win-%VERSION%

REM Create zip archive
powershell Compress-Archive -Path flashsn8-gui-win-%VERSION% -DestinationPath ..\upload\flashsn8-gui-win-%VERSION%.zip -Force

endlocal