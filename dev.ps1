$env:RUSTUP_HOME = "D:\environment\rust\.rustup"
$env:CARGO_HOME = "D:\environment\rust\.cargo"
$env:PATH = "$env:CARGO_HOME\bin;$env:PATH"
pnpm tauri dev
