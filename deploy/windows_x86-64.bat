@echo off
setlocal

REM === 1. Tailwind CSS compile ===
echo Compiling TailwindCSS...
call npx tailwindcss -i ./input.css -o ./assets/tailwind.css --minify
IF ERRORLEVEL 1 (
    echo Tailwind compilation failed.
    exit /b 1
)

REM === 2. Dioxus build ===
echo Building Dioxus app for desktop...
dx build --release --platform desktop
IF ERRORLEVEL 1 (
    echo Dioxus build failed.
    exit /b 1
)

REM === 3. Prepare directories ===
echo Collecting files...
set OUTDIR=deploy\windows_working
rmdir /s /q %OUTDIR%
mkdir %OUTDIR%
set DISTDIR=deploy\windows
rmdir /s /q %DISTDIR%
mkdir %DISTDIR%

REM === 4. Copy directories ===
for %%F in (boards examples firmware logical_layouts settings) do (
    xcopy /s /e /y %%F %OUTDIR%\%%F\
)

REM === 5. Copy built products ===
xcopy /s /e /y target\dx\ku1255-firmware-modifier\release\windows\app\ku1255-firmware-modifier.exe %OUTDIR%\
xcopy /s /e /y target\dx\ku1255-firmware-modifier\release\windows\app\assets\ %OUTDIR%\assets\

REM === 6. Create zip archive ===
echo Creating ZIP archive...
powershell -Command "Compress-Archive -Path %OUTDIR%\* -DestinationPath %DISTDIR%\ku1255-firmware-modifier-windows.zip"
IF NOT EXIST build.zip (
    echo ZIP compression failed.
    exit /b 1
)

echo Done.
endlocal
pause
