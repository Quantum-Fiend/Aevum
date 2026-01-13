@echo off
REM Aevum Startup Script for Windows
REM Starts all Aevum components for development

echo.
echo ═══════════════════════════════════════════
echo    Aevum - Time-Travel Debugging Platform
echo ═══════════════════════════════════════════
echo.

set PROJECT_ROOT=%~dp0..

REM Start Coordinator
echo [INFO] Starting Coordinator...
cd /d "%PROJECT_ROOT%\coordinator"
if not exist "aevum-coordinator.exe" (
    echo [INFO] Building coordinator...
    go build -o aevum-coordinator.exe
)
start "Aevum Coordinator" /B aevum-coordinator.exe
timeout /t 2 /nobreak > nul
echo [OK] Coordinator started on :9876 (trace) and :8080 (API)

REM Start UI
echo [INFO] Starting UI...
cd /d "%PROJECT_ROOT%\ui"
if not exist "node_modules" (
    echo [INFO] Installing UI dependencies...
    call npm install
)
start "Aevum UI" /B npm run dev
timeout /t 3 /nobreak > nul
echo [OK] UI started on http://localhost:3000

echo.
echo ═══════════════════════════════════════════
echo    Aevum is running!
echo ═══════════════════════════════════════════
echo.
echo    Coordinator (Trace): localhost:9876
echo    Coordinator (API):   localhost:8080
echo    UI Dashboard:        http://localhost:3000
echo.
echo    Press any key to stop all services...
echo.

pause > nul

REM Cleanup
taskkill /FI "WINDOWTITLE eq Aevum*" /F > nul 2>&1
echo [OK] Aevum stopped
