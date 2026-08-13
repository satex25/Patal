@echo off
setlocal EnableDelayedExpansion
REM ---------------------------------------------------------------------------
REM Run cargo inside an MSVC developer environment, from anywhere.
REM
REM Why this exists: on Windows the MSVC linker is `link.exe`, and Git Bash
REM ships a coreutils `link` that shadows it on PATH. cargo then fails with
REM
REM     /usr/bin/link: extra operand '/NOLOGO'
REM
REM and rustc's own hint ("the msvc targets depend on the msvc linker...
REM repair your Visual Studio installation") sends you after a problem you do
REM not have. Sourcing vcvars64.bat puts the real link.exe ahead of it.
REM
REM This deliberately does NOT live in .cargo/config.toml: the vcvars path is
REM machine-local and would break CI, which already has a working linker.
REM
REM Usage (from any directory, any shell):
REM     scripts\cargo.bat test --workspace --locked
REM     scripts\cargo.bat clippy --workspace --all-targets --locked -- -D warnings
REM
REM From Git Bash, invoke through cmd so the batch file is interpreted:
REM     cmd //c 'scripts\cargo.bat test --workspace --locked'
REM
REM Defaults to the engine workspace. Override for the Tauri harness:
REM     set PATAL_CARGO_DIR=%~dp0..\apps\desktop\src-tauri
REM ---------------------------------------------------------------------------

if "%PATAL_CARGO_DIR%"=="" set "PATAL_CARGO_DIR=%~dp0..\engine"

set "VSWHERE=%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe"
REM Enter the target directory FIRST, before anything else runs.
REM
REM When this script is launched from a shell whose working directory is a UNC
REM path (Git Bash under \\wsl.localhost\..., for instance), cmd.exe cannot
REM adopt that directory and silently starts somewhere else, printing "The
REM system cannot find the path specified." vcvars64.bat then fails to resolve
REM its own helpers and reports "'vswhere.exe' is not recognized" — an error
REM about a file that is present and a tool that is installed. Moving to a real
REM directory up front makes the whole sequence deterministic.
cd /d "%PATAL_CARGO_DIR%" || (
    echo [patal] Cannot enter "%PATAL_CARGO_DIR%".
    exit /b 1
)

if not exist "%VSWHERE%" (
    echo [patal] vswhere.exe not found at "%VSWHERE%".
    echo [patal] Install Visual Studio Build Tools with the "Desktop development
    echo [patal] with C++" workload, then re-run this script.
    exit /b 1
)

REM Deliberately NOT a `for /f` over the command. vswhere lives under
REM "Program Files (x86)", and the ")" in that path closes the for /f block
REM early no matter how the inner command is quoted or escaped, which shows
REM up as a spurious "'vswhere.exe' is not recognized". Running it as a plain
REM statement and reading the result back avoids the parser entirely.
set "VSINSTALL="
set "VSPROBE=%TEMP%\patal-vsinstall-%RANDOM%.txt"
"%VSWHERE%" -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath > "%VSPROBE%" 2>nul
if exist "%VSPROBE%" (
    set /p VSINSTALL=<"%VSPROBE%"
    del "%VSPROBE%" >nul 2>&1
)

if "%VSINSTALL%"=="" (
    echo [patal] No Visual Studio installation with the MSVC x64 toolset was found.
    echo [patal] Install the "Desktop development with C++" workload.
    exit /b 1
)

set "VCVARS=%VSINSTALL%\VC\Auxiliary\Build\vcvars64.bat"
if not exist "%VCVARS%" (
    echo [patal] Found "%VSINSTALL%" but no vcvars64.bat inside it.
    exit /b 1
)

REM Both streams are suppressed because vcvars64.bat chatters on stderr about
REM its own internals ("'vswhere.exe' is not recognized") while still setting
REM the environment correctly, and that message sends people after a problem
REM they do not have. Its exit code is not trustworthy either, so verify the
REM thing we actually need instead: VCToolsInstallDir is the marker vcvars
REM sets once the MSVC toolset is really on PATH.
call "%VCVARS%" >nul 2>&1

if not defined VCToolsInstallDir (
    echo [patal] vcvars64.bat ran but did not set up the MSVC toolset.
    echo [patal] Tried: "%VCVARS%"
    echo [patal] Re-run the Visual Studio Installer and confirm the
    echo [patal] "Desktop development with C++" workload is installed.
    exit /b 1
)

cargo %*
exit /b %ERRORLEVEL%
