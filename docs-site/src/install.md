# Install

Pick one or both — they're independent, and the extension doesn't require you to
separately install the CLI.

## VS Code / any VS Code-compatible editor (recommended for editing)

Install [Drut for Cube Voyager](https://marketplace.visualstudio.com/items?itemName=arpuuk.drut)
from the VS Code Marketplace, or [from Open VSX](https://open-vsx.org/extension/arpuuk/drut)
on VS Code-compatible editors that use it instead (Cursor, VSCodium, and
similar).

**Nothing else to install.** On first activation, the extension resolves a
working `drut` language server binary automatically: it checks `PATH` first,
then its own persistent extension storage from a prior activation, then — if
neither is present — downloads the correct binary for your platform from the
latest GitHub Release and verifies it against its published SHA-256 checksum
before trusting it. If every option is unavailable (offline, an unsupported
platform, a failed download), the extension degrades to syntax-highlighting-only
rather than failing outright, and tells you why once.

Once installed this way, a throttled (at most once per 24 hours), non-blocking
background check offers a dismissible notification when a newer release is
available — it never silently replaces a running binary.

## Just the CLI (for scripting or CI)

```sh
cargo install drut-cli
```

Or build from source:

```powershell
cargo build --release -p drut-cli
# binary at target/release/drut(.exe) -- put it on PATH
```

Confirm it's working:

```powershell
drut --help
```

Continue to [Getting Started](getting-started.md) to run it against a real
script.
