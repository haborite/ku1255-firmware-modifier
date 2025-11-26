@echo off
setlocal

rem ==========================================
rem Settings
rem ==========================================
set "PY_VERSION=3.8.10"
set "PY_EMBED_URL=https://www.python.org/ftp/python/3.8.10/python-3.8.10-embed-amd64.zip"
set "PY_DIR=python\python-win-embed-amd64"
set "PY_ZIP=%PY_DIR%.zip"
set "SN8_DIR=sn8tools\sn8"

echo === Downloading embeddable Python %PY_VERSION% ===
powershell -Command "Invoke-WebRequest -Uri '%PY_EMBED_URL%' -OutFile '%PY_ZIP%'"

if not exist "%PY_ZIP%" (
    echo [ERROR] Failed to download %PY_ZIP%
    exit /b 1
)

echo === Extracting embeddable Python ===
if exist "%PY_DIR%" (
    echo Removing existing directory "%PY_DIR%"
    rmdir /s /q "%PY_DIR%"
)
powershell -Command "Expand-Archive -Path '%PY_ZIP%' -DestinationPath '%CD%\%PY_DIR%' -Force"
if errorlevel 1 (
    echo [ERROR] Failed to extract %PY_ZIP%
    exit /b 1
)

echo === Enabling site-packages in python38._pth (uncomment import site) ===
powershell -Command "(Get-Content '%PY_DIR%\python38._pth') -replace '^\s*#\s*import site','import site' | Set-Content '%PY_DIR%\python38._pth'"

echo === Downloading get-pip.py ===
powershell -Command "Invoke-WebRequest -Uri 'https://bootstrap.pypa.io/pip/3.8/get-pip.py' -OutFile '%PY_DIR%\get-pip.py'"

if not exist "%PY_DIR%\get-pip.py" (
    echo [ERROR] Failed to download get-pip.py
    exit /b 1
)

pushd "%PY_DIR%"

echo === Installing pip with get-pip.py ===
python.exe get-pip.py
if errorlevel 1 (
    echo [ERROR] get-pip.py failed
    popd
    exit /b 1
)

rem Put this embedded Python and its Scripts dir first on PATH (for this script only)
set "PATH=%CD%;%CD%\Scripts;%PATH%"

echo === Installing required packages (libusb1, ply) ===
python.exe -m pip install --upgrade pip
if errorlevel 1 (
    echo [WARN] pip upgrade failed (continuing)
)
python.exe -m pip install libusb1
if errorlevel 1 (
    echo [ERROR] Failed to install libusb1
    popd
    exit /b 1
)
python.exe -m pip install ply
if errorlevel 1 (
    echo [ERROR] Failed to install ply
    popd
    exit /b 1
)

echo === Cleaning up unnecessary files ===

rem --- remove get-pip script itself
del /f /q "%CD%\get-pip.py" 2>nul

rem --- site-packages directory inside embeddable Python
set "SITE_DIR=%CD%\Lib\site-packages"

rem --- remove pip / setuptools / wheel packages (but keep other libs)
for /d %%D in ("%SITE_DIR%\pip") do if exist "%%D" rd /s /q "%%D"
for /d %%D in ("%SITE_DIR%\pip-*.dist-info") do if exist "%%D" rd /s /q "%%D"

for /d %%D in ("%SITE_DIR%\setuptools") do if exist "%%D" rd /s /q "%%D"
for /d %%D in ("%SITE_DIR%\setuptools-*.dist-info") do if exist "%%D" rd /s /q "%%D"

for /d %%D in ("%SITE_DIR%\wheel") do if exist "%%D" rd /s /q "%%D"
for /d %%D in ("%SITE_DIR%\wheel-*.dist-info") do if exist "%%D" rd /s /q "%%D"

rem --- remove easy_install helpers if present
del /f /q "%SITE_DIR%\easy_install.py" 2>nul
del /f /q "%SITE_DIR%\easy_install-*.py" 2>nul

rem --- remove Scripts\pip, easy_install, wheel wrappers
if exist "%CD%\Scripts" (
    del /f /q "%CD%\Scripts\pip*.exe" 2>nul
    del /f /q "%CD%\Scripts\pip*.bat" 2>nul
    del /f /q "%CD%\Scripts\easy_install*.*" 2>nul
    del /f /q "%CD%\Scripts\wheel*.*" 2>nul
)

rem --- remove tests
if exist "%CD%\Lib\site-packages\pkg_resources\tests" (
    rmdir /s /q "%CD%\Lib\site-packages\pkg_resources\tests"
)

rem --- remove __pycache__ directories recursively (from this embedded tree)
for /f "delims=" %%D in ('dir /ad /b /s "__pycache__" 2^>nul') do (
    rd /s /q "%%D"
)

rem --- (optional) remove wheel files that might be left in site-packages
del /f /q "%SITE_DIR%\*.whl" 2>nul


echo === Removing downloaded ZIP ===
popd
del /f /q "%PY_ZIP%" 2>nul

echo === Copy SN8 ===
xcopy "%SN8_DIR%" "%SITE_DIR%\sn8\" /E /I /H /Y
if errorlevel 1 (
    echo [ERROR] Failed to copy sn8 directory
    exit /b 1
)

echo === Done ===
echo Embeddable Python %PY_VERSION% with libusb1 + ply is ready in:
echo   %CD%

endlocal
