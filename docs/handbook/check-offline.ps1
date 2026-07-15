$ErrorActionPreference = 'Stop'

$handbook = Split-Path -Parent $MyInvocation.MyCommand.Path
$errors = [System.Collections.Generic.List[string]]::new()
$links = 0

Get-ChildItem -LiteralPath $handbook -Filter '*.md' -File | ForEach-Object {
    $document = $_
    $content = Get-Content -LiteralPath $document.FullName -Raw
    [regex]::Matches($content, '\[[^\]]+\]\(([^)]+)\)') | ForEach-Object {
        $target = $_.Groups[1].Value.Trim()
        $links++

        if ($target -match '^[a-zA-Z][a-zA-Z0-9+.-]*:') {
            $errors.Add("$($document.Name): external link is not offline-safe: $target")
            return
        }
        if ($target.StartsWith('#')) {
            return
        }

        $pathPart = ($target -split '#', 2)[0]
        if ([string]::IsNullOrWhiteSpace($pathPart)) {
            return
        }
        $candidate = Join-Path $document.DirectoryName ([uri]::UnescapeDataString($pathPart))
        if (-not (Test-Path -LiteralPath $candidate)) {
            $errors.Add("$($document.Name): missing local target: $target")
        }
    }
}

if ($errors.Count -gt 0) {
    $errors | ForEach-Object { Write-Error $_ }
    exit 1
}

Write-Host "WorldTools handbook is offline-safe: $links local links checked."
