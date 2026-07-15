$ErrorActionPreference = 'Stop'

$handbook = Split-Path -Parent $MyInvocation.MyCommand.Path
$output = Join-Path $handbook 'WorldTools-Handbook.md'
$chapters = @(
    'README.md',
    'getting-started.md',
    'study-plan.md',
    'architecture.md',
    'crate-map.md',
    'design-invariants.md',
    'simulation.md',
    'data-layers.md',
    'rendering-ui.md',
    'debugging.md',
    'reading-tours.md',
    'glossary.md'
)

$parts = [System.Collections.Generic.List[string]]::new()
$parts.Add('# WorldTools Complete Offline Handbook')
$parts.Add('')
$parts.Add('This single-file edition is generated from the maintained chapters beside it.')
$parts.Add('')

foreach ($chapter in $chapters) {
    $path = Join-Path $handbook $chapter
    if (-not (Test-Path -LiteralPath $path)) {
        throw "Missing handbook chapter: $chapter"
    }
    $parts.Add('---')
    $parts.Add('')
    $parts.Add((Get-Content -LiteralPath $path -Raw).Trim())
    $parts.Add('')
}

Set-Content -LiteralPath $output -Value ($parts -join [Environment]::NewLine) -Encoding utf8
Write-Host "Built $output"
