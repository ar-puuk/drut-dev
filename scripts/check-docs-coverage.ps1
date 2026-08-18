<#
.SYNOPSIS
    Fails if any drut-config FormatConfig field is undocumented on the
    published configuration-reference page.

.DESCRIPTION
    022-docs-site (specs/022-docs-site/contracts/config-reference-entry.md):
    every field of `drut_config::FormatConfig` MUST have a `### <field_name>`
    heading in docs-site/src/configuration-reference.md. This is a direct,
    mechanical fix for the exact failure mode that prompted this feature --
    CONTRIBUTING.md's old "Configuration" section silently fell behind and
    only ever documented 2 of the eventual 10 real fields.

    This check only verifies a field's *name* appears as a heading -- it
    can't judge prose quality (values/default/example/precedence
    completeness), which stays a human review concern per
    contracts/config-reference-entry.md's own acceptance checklist.
#>

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$configLib = Join-Path $repoRoot "crates\drut-config\src\lib.rs"
$referencePage = Join-Path $repoRoot "docs-site\src\configuration-reference.md"

if (-not (Test-Path $configLib)) {
    throw "Not found: $configLib"
}
if (-not (Test-Path $referencePage)) {
    throw "Not found: $referencePage"
}

$libText = Get-Content -Raw -LiteralPath $configLib

# Isolate the FormatConfig struct body (from its opening brace to the next
# top-level closing brace) so we only pick up its own fields, not some other
# struct's fields that happen to share a name elsewhere in the file.
$structMatch = [regex]::Match(
    $libText,
    'pub struct FormatConfig \{(?<body>.*?)\n\}',
    [System.Text.RegularExpressions.RegexOptions]::Singleline
)
if (-not $structMatch.Success) {
    throw "Could not find 'pub struct FormatConfig { ... }' in $configLib -- has it been renamed or restructured?"
}
$structBody = $structMatch.Groups["body"].Value

$fieldMatches = [regex]::Matches($structBody, '(?m)^\s*pub (?<name>[a-z_][a-z0-9_]*):')
$fields = $fieldMatches | ForEach-Object { $_.Groups["name"].Value } | Sort-Object -Unique

if ($fields.Count -eq 0) {
    throw "Found FormatConfig but extracted zero field names -- regex likely needs updating."
}

$referenceText = Get-Content -Raw -LiteralPath $referencePage

$missing = @()
foreach ($field in $fields) {
    $headingPattern = "(?m)^###\s+``$([regex]::Escape($field))``\s*$"
    if ($referenceText -notmatch $headingPattern) {
        $missing += $field
    }
}

if ($missing.Count -gt 0) {
    Write-Error "docs-site/src/configuration-reference.md is missing a heading for $($missing.Count) FormatConfig field(s): $($missing -join ', ')"
    exit 1
}

Write-Host "OK: all $($fields.Count) FormatConfig fields ($($fields -join ', ')) have a matching heading in configuration-reference.md"
exit 0
