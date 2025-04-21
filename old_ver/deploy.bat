@echo off
setlocal enabledelayedexpansion

REM Get version as the argument
set VERSION=%1
if not defined VERSION set VERSION=0.4.0

REM Activate virtual environment
REM call C:\path\to\ku1255-env\Scripts\activate.bat

REM Nuitka compile
python -m nuitka --standalone --enable-plugin=pyside6 --windows-console-mode=disable --output-dir=win ku1255-fw-remapper.py

cd win
rename ku1255-fw-remapper.dist ku1255-fw-remapper-win-%VERSION%
copy ..\keymaps.csv ku1255-fw-remapper-win-%VERSION%\keymaps.csv
xcopy ..\examples ku1255-fw-remapper-win-%VERSION%\examples /E /I /H /Y

REM Create zip archive
powershell Compress-Archive -Path ku1255-fw-remapper-win-%VERSION% -DestinationPath ..\upload\ku1255-fw-remapper-win-%VERSION%.zip -Force

endlocal
