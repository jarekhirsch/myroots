# Packs target/release/myroots.exe as MSIX (build with VITE_MYROOTS_STORE=1).
$ErrorActionPreference = 'Stop'
$ver = (Get-Content src-tauri/tauri.conf.json -Raw | ConvertFrom-Json).version + '.0'
$pkg = 'target/msix/pkg'
Remove-Item -Recurse -Force target/msix -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force $pkg | Out-Null
Copy-Item target/release/myroots.exe $pkg/
Copy-Item -Recurse src-tauri/msix/Assets $pkg/Assets
(Get-Content src-tauri/msix/AppxManifest.xml -Raw).Replace('{VERSION}', $ver) | Set-Content -Encoding utf8 $pkg/AppxManifest.xml
$makeappx = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin" -Recurse -Filter makeappx.exe |
  Where-Object { $_.FullName -match '\\x64\\' } | Sort-Object FullName -Descending | Select-Object -First 1
if (-not $makeappx) { throw 'makeappx.exe (Windows SDK) not found' }
$out = "target/msix/MyRootsFamilyTree_${ver}_x64.msix"
& $makeappx.FullName pack /o /d $pkg /p $out
if ($LASTEXITCODE -ne 0) { throw "makeappx failed ($LASTEXITCODE)" }
Write-Output "MSIX: $out"
if ($env:GITHUB_OUTPUT) { "msix=$out" | Out-File -Append -Encoding utf8 $env:GITHUB_OUTPUT }
