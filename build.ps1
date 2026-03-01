param(
    [string]$Dest = "bunnie@10.0.245.194:code/local/"
)

$ErrorActionPreference = "Stop"

function Run-Step {
    param([string]$Description, [scriptblock]$Command)
    Write-Host "`n==> $Description" -ForegroundColor Cyan
    & $Command
    if ($LASTEXITCODE -ne 0) {
        Write-Host "FAILED: $Description (exit code $LASTEXITCODE)" -ForegroundColor Red
        exit $LASTEXITCODE
    }
}

$Root = $PSScriptRoot

# Step 1: Build Zig module
Run-Step "Building Zig module" {
    Set-Location "$Root/src/c"
    python3 -m ziglang build "-Dmodule=dabao_tester"
}

# Step 2: Xous-core cargo xtask builds
Run-Step "Building bao1x-boot1" {
    Set-Location "$Root/xous-core"
    cargo xtask bao1x-boot1 --git-describe v0.10.0
}

Run-Step "Building dabao console" {
    Set-Location "$Root/xous-core"
    cargo xtask dabao dabao-console --no-timestamp --kernel-feature debug-proc --git-describe v0.10.0
}

# Step 3: App build and UF2 packaging
Run-Step "Building dabao-tester-app" {
    Set-Location "$Root"
    cargo build --release --target riscv32imac-unknown-xous-elf --features board-dabao --features bao1x --features utralib/bao1x
}

Run-Step "Packaging UF2" {
    Set-Location "$Root"
    xous-app-uf2 --elf target/riscv32imac-unknown-xous-elf/release/dabao-tester-app
}

# Step 4: Copy artifacts and print MD5s
$Artifacts = @(
    "./xous-core/target/riscv32imac-unknown-none-elf/release/bao1x-boot1.uf2",
    "./xous-core/target/riscv32imac-unknown-xous-elf/release/loader.uf2",
    "./xous-core/target/riscv32imac-unknown-xous-elf/release/xous.uf2",
    "./apps.uf2"
)

Set-Location "$Root"

Write-Host "`n==> Copying artifacts to $Dest" -ForegroundColor Cyan
foreach ($Artifact in $Artifacts) {
    if (-not (Test-Path $Artifact)) {
        Write-Host "ERROR: Artifact not found: $Artifact" -ForegroundColor Red
        exit 1
    }
    Write-Host "  Copying $Artifact"
    scp $Artifact $Dest
    if ($LASTEXITCODE -ne 0) {
        Write-Host "FAILED: scp $Artifact (exit code $LASTEXITCODE)" -ForegroundColor Red
        exit $LASTEXITCODE
    }
}

Write-Host "`n==> MD5 checksums of artifacts:" -ForegroundColor Cyan
foreach ($Artifact in $Artifacts) {
    $Hash = (Get-FileHash $Artifact -Algorithm MD5).Hash
    Write-Host "  $Hash  $Artifact"
}

Write-Host "`nAll steps completed successfully." -ForegroundColor Green