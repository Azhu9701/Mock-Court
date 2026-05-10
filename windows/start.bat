@echo off
chcp 65001 >nul
title 万民幡 Soul Banner

cd /d "%~dp0"

echo ========================================
echo   万民幡 Wan Min Fan
echo   实践与理论的反馈循环
echo ========================================

:: Start API
echo [1/2] 启动 API 服务 (0.0.0.0:3096)...
start "万民幡-API" /B "%~dp0万民幡-api.exe" > api.log 2>&1

:: Wait for API
echo 等待 API 就绪...
:wait_api
timeout /t 2 /nobreak >nul
curl -s http://127.0.0.1:3096/api/v1/health >nul 2>&1
if errorlevel 1 goto :wait_api
echo [1/2] API 就绪 ✓

:: Start Frontend
echo [2/2] 启动前端 (http://localhost:3000)...
cd /d "%~dp0frontend"
start "万民幡-Web" /B node server.js > web.log 2>&1
cd /d "%~dp0"

timeout /t 3 /nobreak >nul

echo.
echo ========================================
echo   API:     http://0.0.0.0:3096
echo   Front:   http://localhost:3000
echo   关闭此窗口将停止所有服务
echo ========================================

start http://localhost:3000

echo 按任意键停止服务...
pause >nul

taskkill /F /FI "WINDOWTITLE eq 万民幡-*" 2>nul
echo 服务已停止
