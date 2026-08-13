//! `file://` URI → real filesystem path conversion
//! (012-toml-configuration/research.md §5) — `lsp_types::Uri` has no
//! `to_file_path()`-style method of its own (confirmed directly against
//! `lsp-types`'s vendored `uri.rs`), so this is hand-rolled here, the one
//! place in `drut-lsp` that needs it.

use std::path::PathBuf;

use crate::document_store::ServerState;

/// Resolves the real on-disk path to use for `drut.toml` discovery for a
/// given document request: the document's own real path if it has one,
/// else the client's workspace root (if any), else `None` (built-in
/// defaults, no discovery attempted) — shared by every handler that calls
/// `drut_config::resolve_format_options` (012-toml-configuration
/// research.md §5, contracts/toml-config-api.md).
pub fn resolve_path(uri: &lsp_types::Uri, state: &ServerState) -> Option<PathBuf> {
    uri_to_path(uri).or_else(|| state.workspace_root().map(|p| p.to_path_buf()))
}

/// Converts a `file://` URI to a real on-disk path. Returns `None` for any
/// non-`file` scheme, or if the path component isn't valid UTF-8 once
/// percent-decoded. Correctly strips the leading `/` before a Windows drive
/// letter (`file:///C:/foo` → `C:\foo`, not `\C:\foo`).
pub fn uri_to_path(uri: &lsp_types::Uri) -> Option<PathBuf> {
    if uri.scheme()?.as_str() != "file" {
        return None;
    }
    let decoded = uri.path().as_estr().decode().into_string().ok()?;
    Some(PathBuf::from(strip_windows_drive_leading_slash(&decoded)))
}

/// `/C:/foo` → `C:/foo`; anything else (POSIX paths, UNC paths without a
/// drive letter) passes through unchanged.
fn strip_windows_drive_leading_slash(decoded: &str) -> &str {
    let bytes = decoded.as_bytes();
    if bytes.len() >= 3 && bytes[0] == b'/' && bytes[1].is_ascii_alphabetic() && bytes[2] == b':' {
        &decoded[1..]
    } else {
        decoded
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn windows_drive_letter_path_strips_the_leading_slash() {
        let uri = lsp_types::Uri::from_str("file:///C:/foo/bar.s").unwrap();
        let path = uri_to_path(&uri).unwrap();
        assert_eq!(path, PathBuf::from("C:/foo/bar.s"));
    }

    #[test]
    fn posix_path_is_unaffected() {
        let uri = lsp_types::Uri::from_str("file:///home/user/a.s").unwrap();
        let path = uri_to_path(&uri).unwrap();
        assert_eq!(path, PathBuf::from("/home/user/a.s"));
    }

    #[test]
    fn non_file_scheme_returns_none() {
        let uri = lsp_types::Uri::from_str("untitled:Untitled-1").unwrap();
        assert!(uri_to_path(&uri).is_none());
    }

    #[test]
    fn percent_encoded_space_is_decoded() {
        let uri = lsp_types::Uri::from_str("file:///C:/My%20Project/a.s").unwrap();
        let path = uri_to_path(&uri).unwrap();
        assert_eq!(path, PathBuf::from("C:/My Project/a.s"));
    }
}
