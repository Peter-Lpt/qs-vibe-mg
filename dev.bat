@echo off
set RUSTUP_HOME=D:\environment\rust\.rustup
set CARGO_HOME=D:\environment\rust\.cargo
set PATH=%CARGO_HOME%\bin;%PATH%
pnpm tauri dev
