@echo off
setlocal

:: Get version as the argument
set PRODUCT_VERSION=%1
if not defined PRODUCT_VERSION set PRODUCT_VERSION=0.5.1

:: Setting
set PYTHON_VERSION=3.10.1
:: set ENV_NAME=nuitka_env
set SCRIPT_NAME=ku1255-fw-remapper.py
set EXE_NAME=ku1255-fw-remapper.exe
set COMPANY_NAME="Haborite"
set PRODUCT_NAME="ku1255-fw-remapper"
set ICON_PATH=..\img\keyboard_5643.ico
set SIGN_CERT=127.0.0.1.crt
set SIGN_KEY=127.0.0.1.key
set SIGN_PFX=127.0.0.1.pfx
set SIGN_TOOL=osslsigncode.exe

:: 仮想環境の作成とアクティベート
:: echo Creating virtual environment...
:: conda create -n %ENV_NAME% python=%PYTHON_VERSION% -y
:: call conda activate %ENV_NAME%

:: 必要パッケージインストール
echo Installing Nuitka...
pip install nuitka

:: Nuitka による .exe 生成
echo Compiling %SCRIPT_NAME%...
python -m nuitka --standalone --onefile ^
    --onefile-tempdir-spec=ssl_server_tempdir ^
    --plugin-enable=pylint-warnings ^
    --enable-plugin=pyside6 ^
    --windows-disable-console ^
    --windows-product-name=%PRODUCT_NAME% ^
    --windows-file-description="Local Server" ^
    --windows-product-version=%PRODUCT_VERSION% ^
    --windows-company-name=%COMPANY_NAME% ^
    --windows-icon-from-ico=%ICON_PATH% ^
    %SCRIPT_NAME%

:: OpenSSL で自己証明書作成
if not exist %SIGN_CERT% (
    echo Creating self-signed certificate...
    openssl req -new -x509 -days 365 -nodes -out %SIGN_CERT% -keyout %SIGN_KEY% ^
        -subj "/CN=127.0.0.1/O=%COMPANY_NAME%/C=JP"
)

:: PFXファイル生成 (osslsigncode用)
if not exist %SIGN_PFX% (
    echo Converting cert to PFX...
    openssl pkcs12 -export -out %SIGN_PFX% -inkey %SIGN_KEY% -in %SIGN_CERT% -passout pass:
)

:: 署名
if exist %SIGN_TOOL% (
    echo Signing executable...
    %SIGN_TOOL% sign -pkcs12 %SIGN_PFX% -pass "" -in dist\%EXE_NAME% -out dist\%EXE_NAME%
) else (
    echo Error: osslsigncode.exe not found!
    exit /b 1
)

echo Build and signing completed.
endlocal
pause
