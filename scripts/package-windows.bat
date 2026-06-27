@echo off
setlocal EnableExtensions
chcp 65001 >nul

set "SCRIPT_DIR=%~dp0"
set "REPO_ROOT=%SCRIPT_DIR%.."
pushd "%REPO_ROOT%" >nul
if errorlevel 1 exit /b 1
for %%I in (.) do set "REPO_ROOT=%%~fI"

set "OUTPUT_DIR=dist-win"
set "BIND=127.0.0.1"
set "PORT=18080"
set "SKIP_UI_INSTALL=0"
set "SKIP_UI_BUILD=0"
set "SKIP_CARGO_BUILD=0"
set "OVERWRITE_CONFIG=0"

:parse_args
if "%~1"=="" goto args_done
if /I "%~1"=="--output" goto arg_output
if /I "%~1"=="--bind" goto arg_bind
if /I "%~1"=="--port" goto arg_port
if /I "%~1"=="--skip-ui-install" goto arg_skip_ui_install
if /I "%~1"=="--skip-ui-build" goto arg_skip_ui_build
if /I "%~1"=="--skip-cargo-build" goto arg_skip_cargo_build
if /I "%~1"=="--overwrite-config" goto arg_overwrite_config
if /I "%~1"=="--help" goto usage
echo [打包] 未知参数：%~1
goto usage

:arg_output
if "%~2"=="" (
    echo [打包] --output 缺少目录参数
    goto usage
)
set "OUTPUT_DIR=%~2"
shift
shift
goto parse_args

:arg_bind
if "%~2"=="" (
    echo [打包] --bind 缺少地址参数
    goto usage
)
set "BIND=%~2"
shift
shift
goto parse_args

:arg_port
if "%~2"=="" (
    echo [打包] --port 缺少端口参数
    goto usage
)
set "PORT=%~2"
shift
shift
goto parse_args

:arg_skip_ui_install
set "SKIP_UI_INSTALL=1"
shift
goto parse_args

:arg_skip_ui_build
set "SKIP_UI_BUILD=1"
shift
goto parse_args

:arg_skip_cargo_build
set "SKIP_CARGO_BUILD=1"
shift
goto parse_args

:arg_overwrite_config
set "OVERWRITE_CONFIG=1"
shift
goto parse_args

:args_done
call :find_yarn
if errorlevel 1 goto fail

call :find_cargo
if errorlevel 1 goto fail

for %%I in ("%OUTPUT_DIR%") do set "OUTPUT_PATH=%%~fI"
for %%I in ("%OUTPUT_PATH%") do set "OUTPUT_ROOT=%%~dI\"

if /I "%OUTPUT_PATH%"=="%REPO_ROOT%" (
    echo [打包] 输出目录不能是仓库根目录：%OUTPUT_PATH%
    goto fail
)
if /I "%OUTPUT_PATH%"=="%OUTPUT_ROOT%" (
    echo [打包] 输出目录不能是磁盘根目录：%OUTPUT_PATH%
    goto fail
)

echo.
echo [打包] 仓库目录：%REPO_ROOT%
echo [打包] 输出目录：%OUTPUT_PATH%

call :install_ui
if errorlevel 1 goto fail

call :build_ui
if errorlevel 1 goto fail

call :build_backend
if errorlevel 1 goto fail

if not exist "%REPO_ROOT%\target\release\web-file-browser.exe" (
    echo [打包] 没有找到后端可执行文件：%REPO_ROOT%\target\release\web-file-browser.exe
    goto fail
)
if not exist "%REPO_ROOT%\ui\dist" (
    echo [打包] 没有找到前端构建目录：%REPO_ROOT%\ui\dist
    goto fail
)

call :assemble_package
if errorlevel 1 goto fail

echo.
echo [打包] 完成
echo [打包] 输出目录：%OUTPUT_PATH%
echo [打包] 启动脚本：%OUTPUT_PATH%\run-windows.bat
echo [打包] 访问地址：http://%BIND%:%PORT%
popd >nul
exit /b 0

:find_yarn
where yarn.cmd >nul 2>nul
if not errorlevel 1 (
    set "YARN=yarn.cmd"
    exit /b 0
)
where yarn >nul 2>nul
if not errorlevel 1 (
    set "YARN=yarn"
    exit /b 0
)
echo [打包] 缺少命令：yarn。请先安装 Node.js 和 Yarn。
exit /b 1

:find_cargo
where cargo.exe >nul 2>nul
if not errorlevel 1 (
    set "CARGO=cargo.exe"
    exit /b 0
)
where cargo >nul 2>nul
if not errorlevel 1 (
    set "CARGO=cargo"
    exit /b 0
)
echo [打包] 缺少命令：cargo。请先安装 Rust。
exit /b 1

:install_ui
if "%SKIP_UI_INSTALL%"=="1" (
    echo.
    echo [打包] 按参数跳过前端依赖安装
    exit /b 0
)
if exist "%REPO_ROOT%\ui\node_modules" if exist "%REPO_ROOT%\ui\.yarn\install-state.gz" (
    echo.
    echo [打包] 前端依赖已存在，跳过 yarn install
    exit /b 0
)
echo.
echo [打包] 安装前端依赖
pushd "%REPO_ROOT%\ui" >nul
if errorlevel 1 exit /b 1
call %YARN% install
set "RESULT=%ERRORLEVEL%"
popd >nul
exit /b %RESULT%

:build_ui
if "%SKIP_UI_BUILD%"=="1" (
    echo.
    echo [打包] 按参数跳过前端构建
    exit /b 0
)
echo.
echo [打包] 构建前端 ui\dist
pushd "%REPO_ROOT%\ui" >nul
if errorlevel 1 exit /b 1
call %YARN% build
set "RESULT=%ERRORLEVEL%"
popd >nul
exit /b %RESULT%

:build_backend
if "%SKIP_CARGO_BUILD%"=="1" (
    echo.
    echo [打包] 按参数跳过后端构建
    exit /b 0
)
echo.
echo [打包] 构建后端 release 可执行文件
pushd "%REPO_ROOT%" >nul
if errorlevel 1 exit /b 1
call %CARGO% build --release
set "RESULT=%ERRORLEVEL%"
popd >nul
exit /b %RESULT%

:assemble_package
echo.
echo [打包] 组装 Windows 测试目录
if not exist "%OUTPUT_PATH%" mkdir "%OUTPUT_PATH%"
if errorlevel 1 exit /b 1
if not exist "%OUTPUT_PATH%\ui" mkdir "%OUTPUT_PATH%\ui"
if errorlevel 1 exit /b 1
if not exist "%OUTPUT_PATH%\data" mkdir "%OUTPUT_PATH%\data"
if errorlevel 1 exit /b 1

copy /Y "%REPO_ROOT%\target\release\web-file-browser.exe" "%OUTPUT_PATH%\web-file-browser.exe" >nul
if errorlevel 1 exit /b 1

if exist "%OUTPUT_PATH%\ui\dist" (
    rmdir /S /Q "%OUTPUT_PATH%\ui\dist"
    if errorlevel 1 exit /b 1
)
xcopy "%REPO_ROOT%\ui\dist" "%OUTPUT_PATH%\ui\dist" /E /I /Y >nul
if errorlevel 1 exit /b 1

if "%OVERWRITE_CONFIG%"=="1" goto write_config
if exist "%OUTPUT_PATH%\data\config.json" goto keep_config

:write_config
call :write_default_config
if errorlevel 1 exit /b 1
echo.
echo [打包] 已写入默认配置：%OUTPUT_PATH%\data\config.json
goto write_extra_files

:keep_config
echo.
echo [打包] 保留已有配置：%OUTPUT_PATH%\data\config.json

:write_extra_files
call :write_run_script
if errorlevel 1 exit /b 1
call :write_readme
if errorlevel 1 exit /b 1
exit /b 0

:write_default_config
>"%OUTPUT_PATH%\data\config.json" echo {
>>"%OUTPUT_PATH%\data\config.json" echo   "server": {
>>"%OUTPUT_PATH%\data\config.json" echo     "bind": "%BIND%",
>>"%OUTPUT_PATH%\data\config.json" echo     "port": %PORT%,
>>"%OUTPUT_PATH%\data\config.json" echo     "staticDir": "ui/dist"
>>"%OUTPUT_PATH%\data\config.json" echo   }
>>"%OUTPUT_PATH%\data\config.json" echo }
exit /b %ERRORLEVEL%

:write_run_script
>"%OUTPUT_PATH%\run-windows.bat" echo @echo off
>>"%OUTPUT_PATH%\run-windows.bat" echo cd /d "%%~dp0"
>>"%OUTPUT_PATH%\run-windows.bat" echo web-file-browser.exe
exit /b %ERRORLEVEL%

:write_readme
>"%OUTPUT_PATH%\README-WINDOWS.txt" echo Web File Browser Windows 测试包
>>"%OUTPUT_PATH%\README-WINDOWS.txt" echo.
>>"%OUTPUT_PATH%\README-WINDOWS.txt" echo 启动方式：
>>"%OUTPUT_PATH%\README-WINDOWS.txt" echo 1. 双击 run-windows.bat，或在命令行执行它。
>>"%OUTPUT_PATH%\README-WINDOWS.txt" echo 2. 浏览器打开：http://%BIND%:%PORT%
>>"%OUTPUT_PATH%\README-WINDOWS.txt" echo.
>>"%OUTPUT_PATH%\README-WINDOWS.txt" echo 说明：
>>"%OUTPUT_PATH%\README-WINDOWS.txt" echo - ui\dist 是前端静态文件目录，不能删除。
>>"%OUTPUT_PATH%\README-WINDOWS.txt" echo - data 是运行数据目录，管理员密码哈希、挂载配置和收藏会保存在这里。
>>"%OUTPUT_PATH%\README-WINDOWS.txt" echo - 首次进入 Web 页面时设置管理员密码。
>>"%OUTPUT_PATH%\README-WINDOWS.txt" echo - 如需改端口，可编辑 data\config.json 后重启。
exit /b %ERRORLEVEL%

:usage
echo 用法：scripts\package-windows.bat [选项]
echo.
echo 选项：
echo   --output DIR           输出目录，默认 dist-win
echo   --bind ADDRESS         监听地址，默认 127.0.0.1
echo   --port PORT            端口，默认 18080
echo   --skip-ui-install      跳过 yarn install
echo   --skip-ui-build        跳过前端构建
echo   --skip-cargo-build     跳过后端构建
echo   --overwrite-config     覆盖输出目录中的 data\config.json
echo   --help                 显示帮助
popd >nul
exit /b 2

:fail
echo.
echo [打包] 失败
popd >nul
exit /b 1
