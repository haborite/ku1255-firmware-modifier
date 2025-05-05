@echo off
setlocal

:: === Settings ===
set "WORKDIR=deploy\windows_working"
set "DISTDIR=deploy\windows"
set "ARCHIVE_NAME=ku1255-firmware-modifier-windows.zip"
set "FLASH_GUI_ZIP=deploy\flashsn8-gui-win.zip"
set "FLASH_GUI_EXTRACTED=deploy\flashsn8-gui-win"
set "FLASH_GUI_DEST=%WORKDIR%\firmware\flashsn8"

:: === 1. Compile TailwindCSS ===
echo === 1. Compiling TailwindCSS ===
call npx tailwindcss -i ./input.css -o ./assets/tailwind.css --minify
if errorlevel 1 (
    echo TailwindCSS compilation failed.
    exit /b 1
)

:: === 2. Build Dioxus App ===
echo === 2. Building Dioxus App ===
dx build --release --platform desktop
if errorlevel 1 (
    echo Dioxus build failed.
    exit /b 1
)

:: === 3. Prepare Output Directories ===
echo === 3. Preparing Output Directories ===
rmdir /s /q "%WORKDIR%" 2>nul
mkdir "%WORKDIR%"
rmdir /s /q "%DISTDIR%" 2>nul
mkdir "%DISTDIR%"

:: === 4. Download flashsn8-gui if not cached ===
echo === 4. Downloading flashsn8-gui (cached if exists) ===
if not exist "%FLASH_GUI_ZIP%" (
    powershell -Command "Invoke-WebRequest -Uri 'https://github.com/haborite/flashsn8-gui/releases/download/0.0.1/flashsn8-gui-win-0.0.1.zip' -OutFile '%FLASH_GUI_ZIP%'"
) else (
    echo Using cached flashsn8-gui ZIP file: %FLASH_GUI_ZIP%
)

:: === 5. Extract flashsn8-gui ===
echo === 5. Extracting flashsn8-gui ===
rmdir /s /q "%FLASH_GUI_EXTRACTED%" 2>nul
powershell -Command "Expand-Archive -Path '%FLASH_GUI_ZIP%' -DestinationPath '%FLASH_GUI_EXTRACTED%'"

:: === 6. Copy extracted files into firmware\flashsn8 ===
echo === 6. Copying flashsn8-gui contents to firmware directory ===
rmdir /s /q "%FLASH_GUI_DEST%" 2>nul
mkdir "%FLASH_GUI_DEST%"
xcopy /e /i /y "%FLASH_GUI_EXTRACTED%\flashsn8-gui-win-0.0.1\*" "%FLASH_GUI_DEST%\" >nul
rmdir /s /q "%FLASH_GUI_EXTRACTED%" 2>nul

:: === 7. Copy project directories ===
echo === 7. Copying project resources ===
for %%F in (boards examples logical_layouts settings) do (
    xcopy /s /e /y "%%F" "%WORKDIR%\%%F\" >nul
)

:: === 8. Copy built binary and assets ===
echo === 8. Copying built artifacts ===
xcopy /s /e /y "target\dx\ku1255-firmware-modifier\release\windows\app\ku1255-firmware-modifier.exe" "%WORKDIR%\" >nul
xcopy /s /e /y "target\dx\ku1255-firmware-modifier\release\windows\app\assets\" "%WORKDIR%\assets\" >nul

:: === 9. Create ZIP archive ===
echo === 9. Creating ZIP archive ===
powershell -Command "Compress-Archive -Path '%WORKDIR%\*' -DestinationPath '%DISTDIR%\%ARCHIVE_NAME%'"
if not exist "%DISTDIR%\%ARCHIVE_NAME%" (
    echo ZIP compression failed.
    exit /b 1
)

echo === Build Complete ===
endlocal
