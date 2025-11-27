@echo off
setlocal enabledelayedexpansion

REM ==========================================
REM User settings
REM ==========================================
REM App name and entry script
set "APP_NAME=sn8tool"
set "ENTRY_SCRIPT=%APP_NAME%.py"

REM Project root directory
set "PROJECT_ROOT=%~dp0"

REM config.yaml path
set "CONFIG_YAML=%PROJECT_ROOT%config.yaml"

REM venv directory
set "VENV_DIR=%PROJECT_ROOT%.venv_%APP_NAME%"

REM Python launcher command
set "PY_LAUNCHER=py -3"
set "FALLBACK_PYTHON=python"

REM Build / output directories
set "BUILD_DIR=%PROJECT_ROOT%win"
:: set "UPLOAD_DIR=%PROJECT_ROOT%upload"

REM Nuitka build settings
set "DIST_DIR_NAME=%APP_NAME%.dist"
set "OUTPUT_DIR_NAME=%APP_NAME%"

REM Nuitka common options
set "NUITKA_COMMON_OPTS=--standalone --enable-plugin=pyside6 --no-deployment-flag=self-execution --windows-console-mode=disable"
:: set "NUITKA_COMMON_OPTS=--standalone --enable-plugin=pyside6 --no-deployment-flag=self-execution"

REM Python dependencies to install
set "PY_DEPS=nuitka libusb1 ply PySide6"

REM sn8 cfg settings
set "CFG_REL_DIR=sn8"
set "CFG_FILE=sn8f2288.cfg"

REM ==========================================
REM Main flow
REM ==========================================
cd /d "%PROJECT_ROOT%"

call :prepare_dirs || exit /b 1
call :ensure_venv   || exit /b 1
call :install_deps  || exit /b 1
call :build_app     || exit /b 1
call :package_app   || exit /b 1

echo.
echo === SUCCESS ===
echo.

endlocal
exit /b 0

REM ==========================================
REM Subroutines
REM ==========================================

:prepare_dirs
echo === Preparing directories ===
if not exist "%BUILD_DIR%" (
    mkdir "%BUILD_DIR%" || (call :fail "Failed to create BUILD_DIR" & exit /b 1)
)
:: if not exist "%UPLOAD_DIR%" (
::     mkdir "%UPLOAD_DIR%" || (call :fail "Failed to create UPLOAD_DIR" & exit /b 1)
:: )
exit /b 0

:ensure_venv
echo === Ensuring virtual environment ===
if exist "%VENV_DIR%\Scripts\python.exe" (
    echo   Using existing venv: "%VENV_DIR%"
    set "VENV_PY=%VENV_DIR%\Scripts\python.exe"
    exit /b 0
)

echo   Creating new venv: "%VENV_DIR%"
%PY_LAUNCHER% -m venv "%VENV_DIR%" 2>nul
if errorlevel 1 (
    %FALLBACK_PYTHON% -m venv "%VENV_DIR%"
    if errorlevel 1 (
        call :fail "Failed to create virtual environment"
        exit /b 1
    )
)

set "VENV_PY=%VENV_DIR%\Scripts\python.exe"
exit /b 0

:install_deps
echo === Installing / updating build dependencies ===
set "VENV_PY=%VENV_DIR%\Scripts\python.exe"

"%VENV_PY%" -m pip install --upgrade pip
if errorlevel 1 (
    call :fail "Failed to upgrade pip"
    exit /b 1
)

"%VENV_PY%" -m pip install --upgrade %PY_DEPS%
if errorlevel 1 (
    call :fail "Failed to install Python build dependencies (%PY_DEPS%)"
    exit /b 1
)
exit /b 0

:build_app
echo === Building %APP_NAME% with Nuitka ===

if not exist "%ENTRY_SCRIPT%" (
    call :fail "Entry script not found: %ENTRY_SCRIPT%"
    exit /b 1
)

REM Get entry script directory
for %%F in ("%ENTRY_SCRIPT%") do set "ENTRY_DIR=%%~dpF"

if not exist "%CONFIG_YAML%" (
    echo [WARN] config.yaml not found at: %CONFIG_YAML%
    echo        Nuitka will be run without --user-package-configuration-file
    set "NUITKA_CONFIG_OPT="
) else (
    set "NUITKA_CONFIG_OPT=--user-package-configuration-file=%CONFIG_YAML%"
)

set "VENV_PY=%VENV_DIR%\Scripts\python.exe"

"%VENV_PY%" -m nuitka %NUITKA_COMMON_OPTS% %NUITKA_CONFIG_OPT% ^
    --output-dir="%BUILD_DIR%" ^
    "%ENTRY_SCRIPT%"

if errorlevel 1 (
    call :fail "Nuitka build failed"
    exit /b 1
)

exit /b 0

:package_app
echo === Packaging build results ===
cd /d "%BUILD_DIR%"

if exist "%OUTPUT_DIR_NAME%" (
    echo   Removing old directory: %OUTPUT_DIR_NAME%
    rmdir /s /q "%OUTPUT_DIR_NAME%"
)

if not exist "%DIST_DIR_NAME%" (
    call :fail "Dist directory not found: %DIST_DIR_NAME%"
    exit /b 1
)

echo   Renaming "%DIST_DIR_NAME%" → "%OUTPUT_DIR_NAME%"
rename "%DIST_DIR_NAME%" "%OUTPUT_DIR_NAME%"
if errorlevel 1 (
    call :fail "Failed to rename dist directory"
    exit /b 1
)

REM ------------------------------------------
REM Copy sn8/sn8f2288.cfg to output directory
REM ------------------------------------------
set "CFG_SRC=%ENTRY_DIR%%CFG_REL_DIR%\%CFG_FILE%"
set "CFG_DST_DIR=%OUTPUT_DIR_NAME%\%CFG_REL_DIR%"
set "CFG_DST=%CFG_DST_DIR%\%CFG_FILE%"

echo   Copying cfg: "%CFG_SRC%" → "%CFG_DST%"

if not exist "%CFG_SRC%" (
    echo [WARN] Config not found at: %CFG_SRC%
) else (
    if not exist "%CFG_DST_DIR%" (
        mkdir "%CFG_DST_DIR%" || (call :fail "Failed to create cfg dest dir" & exit /b 1)
    )
    copy /y "%CFG_SRC%" "%CFG_DST%" >nul
    if errorlevel 1 (
        call :fail "Failed to copy cfg file"
        exit /b 1
    )
)

:: echo   Creating ZIP: %OUTPUT_ZIP%
:: powershell -Command "Compress-Archive -Path '%OUTPUT_DIR_NAME%' -DestinationPath '%OUTPUT_ZIP%' -Force"
:: if errorlevel 1 (
::     call :fail "Failed to create ZIP archive"
::     exit /b 1
:: )

exit /b 0

:fail
echo [ERROR] %~1
exit /b 1
