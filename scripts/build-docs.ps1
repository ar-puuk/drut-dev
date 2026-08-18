<#
.SYNOPSIS
    Builds the docs-site mdBook and stamps the committed output with .nojekyll.

.DESCRIPTION
    022-docs-site: mdBook's `book.toml` redirects build output to repo-root
    `docs/` (build-dir = "../docs"), which GitHub Pages ("Deploy from a
    branch" -> main -> /docs) serves directly -- no GitHub Actions deploy
    step (specs/022-docs-site/research.md SS2). mdBook clears its build
    directory on every build, so `docs/.nojekyll` (which disables GitHub's
    default Jekyll processing of the /docs folder -- needed for any
    non-Jekyll static site served this way) has to be recreated after each
    build rather than committed once and expected to survive.

    This script is the single source of truth for "how the site gets
    built" -- used identically by a maintainer publishing locally
    (specs/022-docs-site/quickstart.md step 5) and by
    .github/workflows/docs.yml's freshness-check step, so both always build
    the exact same way.
#>

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$docsSite = Join-Path $repoRoot "docs-site"

if (-not (Test-Path $docsSite)) {
    throw "docs-site/ not found at $docsSite"
}

Push-Location $docsSite
try {
    mdbook build
    if ($LASTEXITCODE -ne 0) {
        throw "mdbook build failed with exit code $LASTEXITCODE"
    }
}
finally {
    Pop-Location
}

$docsOut = Join-Path $repoRoot "docs"
if (-not (Test-Path $docsOut)) {
    throw "Expected build output at $docsOut -- check docs-site/book.toml's [build] build-dir"
}

$nojekyll = Join-Path $docsOut ".nojekyll"
if (-not (Test-Path $nojekyll)) {
    New-Item -ItemType File -Path $nojekyll | Out-Null
}

Write-Host "Built docs-site -> $docsOut (with .nojekyll)"
