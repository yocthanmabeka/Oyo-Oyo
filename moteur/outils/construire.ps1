# Construit le moteur pour le navigateur : tests, compilation WebAssembly, paquet web/pkg.
#     .\outils\construire.ps1
$ErrorActionPreference = "Stop"
$moteur = Split-Path -Parent $PSScriptRoot
Set-Location $moteur

$version = "0.2.100"   # doit être celle de wasm-bindgen dans Cargo.toml
$bin = Join-Path $PSScriptRoot "bin"
$wb = Join-Path $bin "wasm-bindgen-$version-x86_64-pc-windows-msvc\wasm-bindgen.exe"
if (-not (Test-Path $wb)) {
    New-Item -ItemType Directory -Force $bin | Out-Null
    $archive = Join-Path $bin "wasm-bindgen.tar.gz"
    Write-Host "Téléchargement de wasm-bindgen $version..."
    Invoke-WebRequest -Uri "https://github.com/rustwasm/wasm-bindgen/releases/download/$version/wasm-bindgen-$version-x86_64-pc-windows-msvc.tar.gz" -OutFile $archive
    tar -xzf $archive -C $bin
    Remove-Item $archive
}

cargo test
cargo build --release --target wasm32-unknown-unknown
& $wb --target web --no-typescript --out-dir web/pkg target/wasm32-unknown-unknown/release/holo_moteur.wasm

$wasm = Get-Item web/pkg/holo_moteur_bg.wasm
$brotli = node -e "const z=require('zlib'),fs=require('fs');process.stdout.write(String(z.brotliCompressSync(fs.readFileSync('web/pkg/holo_moteur_bg.wasm'),{params:{[z.constants.BROTLI_PARAM_QUALITY]:11}}).length))"
Write-Host ("Moteur : {0:N0} Ko réels, {1:N0} Ko transférés (Brotli)" -f ($wasm.Length / 1KB), ([int]$brotli / 1KB))
Write-Host "Lancer : node outils/serveur.mjs"
