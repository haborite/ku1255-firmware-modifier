@echo off
setlocal

:: === Settings ===
set "WORKDIR=deploy\windows_working"
set "DISTDIR=deploy\windows"
set "ARCHIVE_NAME=ku1255-firmware-modifier-windows.zip"

:: === 0. Build sn8 tools ===
call sn8tool\build-win.bat

:: === 1. Compile TailwindCSS ===
echo === 1. Compiling TailwindCSS ===
call npm install
call npx tailwindcss -i ./input.css -o ./public/tailwind.css --minify
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

:: === 7. Copy project directories ===
echo === 7. Copying project resources ===
for %%F in (boards examples logical_layouts settings template firmware) do (
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
