//! A safe wrapper over the Win32 registry API, and the uninstall-key scanner.
//!
//! Two things here are easy to get wrong and are therefore handled once:
//!
//! * **Registry views.** A 64-bit process sees 32-bit software under
//!   `WOW6432Node`. Opening the same logical path twice, once with
//!   `KEY_WOW64_64KEY` and once with `KEY_WOW64_32KEY`, is the only way to
//!   enumerate everything — miss it and half the installed programs on a
//!   typical machine are invisible.
//! * **Handle lifetime.** [`RegKey`] closes its handle on drop, so an early
//!   return in the middle of a deep enumeration cannot leak one.

use crate::wide::{from_wide, WideString};
use cwico_core::{Error, Result};
use windows::core::PCWSTR;
use windows::Win32::Foundation::{ERROR_MORE_DATA, ERROR_NO_MORE_ITEMS, ERROR_SUCCESS};
use windows::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExW, RegDeleteTreeW, RegDeleteValueW, RegEnumKeyExW, RegEnumValueW,
    RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY, HKEY_CLASSES_ROOT, HKEY_CURRENT_CONFIG,
    HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, HKEY_USERS, KEY_ENUMERATE_SUB_KEYS, KEY_QUERY_VALUE,
    KEY_READ, KEY_SET_VALUE, KEY_WOW64_32KEY, KEY_WOW64_64KEY, KEY_WRITE, REG_BINARY, REG_DWORD,
    REG_EXPAND_SZ, REG_MULTI_SZ, REG_OPTION_NON_VOLATILE, REG_QWORD, REG_SAM_FLAGS, REG_SZ,
    REG_VALUE_TYPE,
};

/// Which of the two registry views to use for 32/64-bit redirection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegView {
    /// The native 64-bit view.
    Bits64,
    /// The `WOW6432Node` view where 32-bit installers write.
    Bits32,
}

impl RegView {
    fn sam(self) -> REG_SAM_FLAGS {
        match self {
            RegView::Bits64 => KEY_WOW64_64KEY,
            RegView::Bits32 => KEY_WOW64_32KEY,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            RegView::Bits64 => "64",
            RegView::Bits32 => "32",
        }
    }
}

/// A decoded registry value.
#[derive(Debug, Clone, PartialEq)]
pub enum RegValue {
    Str(String),
    ExpandStr(String),
    MultiStr(Vec<String>),
    Dword(u32),
    Qword(u64),
    Binary(Vec<u8>),
    Other(u32),
}

impl RegValue {
    /// The value as a string, for the four types that have a sensible one.
    pub fn as_string(&self) -> Option<String> {
        match self {
            RegValue::Str(s) | RegValue::ExpandStr(s) => Some(s.clone()),
            RegValue::MultiStr(v) => Some(v.join("\n")),
            RegValue::Dword(n) => Some(n.to_string()),
            RegValue::Qword(n) => Some(n.to_string()),
            _ => None,
        }
    }

    pub fn as_u32(&self) -> Option<u32> {
        match self {
            RegValue::Dword(n) => Some(*n),
            RegValue::Qword(n) => u32::try_from(*n).ok(),
            RegValue::Str(s) | RegValue::ExpandStr(s) => s.trim().parse().ok(),
            _ => None,
        }
    }
}

/// Parse `HKLM`, `HKEY_LOCAL_MACHINE`, … into a predefined key handle plus the
/// remaining subkey path.
pub fn split_hive(path: &str) -> Option<(HKEY, &str)> {
    let (hive, rest) = match path.split_once('\\') {
        Some((h, r)) => (h, r),
        None => (path, ""),
    };
    let key = match hive.to_ascii_uppercase().as_str() {
        "HKLM" | "HKEY_LOCAL_MACHINE" => HKEY_LOCAL_MACHINE,
        "HKCU" | "HKEY_CURRENT_USER" => HKEY_CURRENT_USER,
        "HKCR" | "HKEY_CLASSES_ROOT" => HKEY_CLASSES_ROOT,
        "HKU" | "HKEY_USERS" => HKEY_USERS,
        "HKCC" | "HKEY_CURRENT_CONFIG" => HKEY_CURRENT_CONFIG,
        _ => return None,
    };
    Some((key, rest))
}

/// An open registry key that closes itself.
pub struct RegKey {
    handle: HKEY,
    /// Kept for error messages: a bare `ERROR_ACCESS_DENIED` is useless
    /// without knowing which key produced it.
    path: String,
}

impl std::fmt::Debug for RegKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegKey").field("path", &self.path).finish()
    }
}

impl Drop for RegKey {
    fn drop(&mut self) {
        if !self.handle.is_invalid() {
            // SAFETY: `handle` came from a successful Reg*KeyEx call and is
            // closed exactly once, here.
            unsafe {
                let _ = RegCloseKey(self.handle);
            }
        }
    }
}

impl RegKey {
    /// Open an existing key for reading.
    pub fn open_read(root: HKEY, subkey: &str, view: RegView) -> Result<Self> {
        Self::open(root, subkey, view, KEY_READ)
    }

    /// Open an existing key for reading and writing.
    pub fn open_write(root: HKEY, subkey: &str, view: RegView) -> Result<Self> {
        Self::open(root, subkey, view, KEY_READ | KEY_WRITE)
    }

    pub fn open(root: HKEY, subkey: &str, view: RegView, access: REG_SAM_FLAGS) -> Result<Self> {
        let wide = WideString::new(subkey);
        let mut handle = HKEY::default();
        // SAFETY: `wide` outlives the call; `handle` is a valid out-pointer.
        let status = unsafe {
            RegOpenKeyExW(
                root,
                wide.as_pcwstr(),
                None,
                access | view.sam(),
                &mut handle,
            )
        };
        if status != ERROR_SUCCESS {
            return Err(Error::Registry {
                key: subkey.to_string(),
                source_msg: format!("RegOpenKeyExW failed with {}", status.0),
            });
        }
        Ok(Self {
            handle,
            path: subkey.to_string(),
        })
    }

    /// Open a key by full path (`HKLM\Software\...`), creating nothing.
    pub fn open_path(full_path: &str, view: RegView, access: REG_SAM_FLAGS) -> Result<Self> {
        let (root, rest) = split_hive(full_path).ok_or_else(|| Error::Registry {
            key: full_path.to_string(),
            source_msg: "unrecognised registry hive".into(),
        })?;
        Self::open(root, rest, view, access)
    }

    /// Open or create a key by full path.
    pub fn create_path(full_path: &str, view: RegView) -> Result<Self> {
        let (root, rest) = split_hive(full_path).ok_or_else(|| Error::Registry {
            key: full_path.to_string(),
            source_msg: "unrecognised registry hive".into(),
        })?;
        let wide = WideString::new(rest);
        let mut handle = HKEY::default();
        // SAFETY: out-pointer is valid; `wide` outlives the call.
        let status = unsafe {
            RegCreateKeyExW(
                root,
                wide.as_pcwstr(),
                None,
                PCWSTR::null(),
                REG_OPTION_NON_VOLATILE,
                KEY_READ | KEY_WRITE | view.sam(),
                None,
                &mut handle,
                None,
            )
        };
        if status != ERROR_SUCCESS {
            return Err(Error::Registry {
                key: full_path.to_string(),
                source_msg: format!("RegCreateKeyExW failed with {}", status.0),
            });
        }
        Ok(Self {
            handle,
            path: full_path.to_string(),
        })
    }

    pub fn handle(&self) -> HKEY {
        self.handle
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    /// Names of the immediate subkeys.
    pub fn subkey_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        let mut index = 0u32;
        loop {
            // Key names are capped at 255 characters by the registry itself.
            let mut buf = [0u16; 256];
            let mut len = buf.len() as u32;
            // SAFETY: `buf`/`len` describe a buffer the API may fill; the
            // handle is open for enumeration.
            let status = unsafe {
                RegEnumKeyExW(
                    self.handle,
                    index,
                    Some(windows::core::PWSTR(buf.as_mut_ptr())),
                    &mut len,
                    None,
                    None,
                    None,
                    None,
                )
            };
            if status == ERROR_NO_MORE_ITEMS {
                break;
            }
            if status != ERROR_SUCCESS {
                tracing::debug!(
                    key = %self.path,
                    index,
                    code = status.0,
                    "RegEnumKeyExW stopped early"
                );
                break;
            }
            names.push(from_wide(&buf[..len as usize]));
            index += 1;
        }
        names
    }

    /// Read one value by name.
    pub fn value(&self, name: &str) -> Option<RegValue> {
        let wide = WideString::new(name);
        let mut kind = REG_VALUE_TYPE::default();
        let mut size = 0u32;

        // First call sizes the buffer.
        // SAFETY: null data pointer with a valid size out-pointer is the
        // documented way to query the required length.
        let status = unsafe {
            RegQueryValueExW(
                self.handle,
                wide.as_pcwstr(),
                None,
                Some(&mut kind),
                None,
                Some(&mut size),
            )
        };
        if status != ERROR_SUCCESS && status != ERROR_MORE_DATA {
            return None;
        }

        let mut buf = vec![0u8; size as usize];
        // SAFETY: `buf` is exactly `size` bytes, which the call above reported.
        let status = unsafe {
            RegQueryValueExW(
                self.handle,
                wide.as_pcwstr(),
                None,
                Some(&mut kind),
                Some(buf.as_mut_ptr()),
                Some(&mut size),
            )
        };
        if status != ERROR_SUCCESS {
            return None;
        }
        buf.truncate(size as usize);
        Some(decode_value(kind, &buf))
    }

    /// Convenience: read a string value, treating empty as absent.
    pub fn string(&self, name: &str) -> Option<String> {
        self.value(name)
            .and_then(|v| v.as_string())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    /// Convenience: read a numeric value.
    pub fn u32(&self, name: &str) -> Option<u32> {
        self.value(name).and_then(|v| v.as_u32())
    }

    /// Every value in this key, as `(name, value)`.
    pub fn values(&self) -> Vec<(String, RegValue)> {
        let mut out = Vec::new();
        let mut index = 0u32;
        loop {
            // Value names are capped at 16383 characters.
            let mut name_buf = [0u16; 16_384];
            let mut name_len = name_buf.len() as u32;
            let mut kind = 0u32;
            let mut data_len = 0u32;

            // SAFETY: buffers and lengths are consistent; data pointer is null
            // on this pass so only the size is reported.
            let status = unsafe {
                RegEnumValueW(
                    self.handle,
                    index,
                    Some(windows::core::PWSTR(name_buf.as_mut_ptr())),
                    &mut name_len,
                    None,
                    Some(&mut kind),
                    None,
                    Some(&mut data_len),
                )
            };
            if status == ERROR_NO_MORE_ITEMS {
                break;
            }
            if status != ERROR_SUCCESS && status != ERROR_MORE_DATA {
                break;
            }

            let name = from_wide(&name_buf[..name_len as usize]);
            if let Some(value) = self.value(&name) {
                out.push((name, value));
            }
            index += 1;
        }
        out
    }

    /// Write a value.
    pub fn set_value(&self, name: &str, value: &RegValue) -> Result<()> {
        let wide_name = WideString::new(name);
        let (kind, bytes) = encode_value(value);
        // SAFETY: `bytes` is a live slice for the duration of the call.
        let status = unsafe {
            RegSetValueExW(
                self.handle,
                wide_name.as_pcwstr(),
                None,
                kind,
                Some(bytes.as_slice()),
            )
        };
        if status != ERROR_SUCCESS {
            return Err(Error::Registry {
                key: format!("{}::{name}", self.path),
                source_msg: format!("RegSetValueExW failed with {}", status.0),
            });
        }
        Ok(())
    }

    /// Delete a value. A value that is already absent is a success, so that
    /// re-running a plan is a no-op rather than a wall of errors.
    pub fn delete_value(&self, name: &str) -> Result<bool> {
        let wide = WideString::new(name);
        // SAFETY: `wide` outlives the call.
        let status = unsafe { RegDeleteValueW(self.handle, wide.as_pcwstr()) };
        match status {
            s if s == ERROR_SUCCESS => Ok(true),
            // ERROR_FILE_NOT_FOUND
            s if s.0 == 2 => Ok(false),
            s => Err(Error::Registry {
                key: format!("{}::{name}", self.path),
                source_msg: format!("RegDeleteValueW failed with {}", s.0),
            }),
        }
    }
}

/// Delete a key and everything under it.
///
/// The caller **must** have run [`cwico_core::guard::validate_delete_key`]
/// first; this function does not re-check, because it is also used to remove
/// subkeys of an already-validated tree.
pub fn delete_tree(full_path: &str, view: RegView) -> Result<bool> {
    let (root, rest) = split_hive(full_path).ok_or_else(|| Error::Registry {
        key: full_path.to_string(),
        source_msg: "unrecognised registry hive".into(),
    })?;

    // Open the parent so `RegDeleteTreeW` removes the key itself, not just
    // its children.
    let (parent_path, leaf) = match rest.rsplit_once('\\') {
        Some((p, l)) => (p, l),
        None => {
            return Err(Error::Registry {
                key: full_path.to_string(),
                source_msg: "refusing to delete a key directly under a hive root".into(),
            })
        }
    };

    let parent = match RegKey::open(root, parent_path, view, KEY_READ | KEY_WRITE) {
        Ok(k) => k,
        // Parent gone means the target is gone: idempotent success.
        Err(_) => return Ok(false),
    };

    let wide_leaf = WideString::new(leaf);
    // SAFETY: `parent` is open for writing and `wide_leaf` outlives the call.
    let status = unsafe { RegDeleteTreeW(parent.handle, wide_leaf.as_pcwstr()) };
    match status {
        s if s == ERROR_SUCCESS => Ok(true),
        // ERROR_FILE_NOT_FOUND: already gone.
        s if s.0 == 2 => Ok(false),
        s => Err(Error::Registry {
            key: full_path.to_string(),
            source_msg: format!("RegDeleteTreeW failed with {}", s.0),
        }),
    }
}

fn decode_value(kind: REG_VALUE_TYPE, data: &[u8]) -> RegValue {
    let as_utf16 = || -> Vec<u16> {
        data.chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect()
    };

    match kind {
        REG_SZ => RegValue::Str(from_wide(&as_utf16())),
        REG_EXPAND_SZ => RegValue::ExpandStr(from_wide(&as_utf16())),
        REG_MULTI_SZ => {
            let units = as_utf16();
            RegValue::MultiStr(
                units
                    .split(|&c| c == 0)
                    .filter(|s| !s.is_empty())
                    .map(String::from_utf16_lossy)
                    .collect(),
            )
        }
        REG_DWORD => {
            let mut b = [0u8; 4];
            let n = data.len().min(4);
            b[..n].copy_from_slice(&data[..n]);
            RegValue::Dword(u32::from_le_bytes(b))
        }
        REG_QWORD => {
            let mut b = [0u8; 8];
            let n = data.len().min(8);
            b[..n].copy_from_slice(&data[..n]);
            RegValue::Qword(u64::from_le_bytes(b))
        }
        REG_BINARY => RegValue::Binary(data.to_vec()),
        other => RegValue::Other(other.0),
    }
}

fn encode_value(value: &RegValue) -> (REG_VALUE_TYPE, Vec<u8>) {
    fn utf16_bytes(s: &str) -> Vec<u8> {
        s.encode_utf16()
            .chain(std::iter::once(0))
            .flat_map(u16::to_le_bytes)
            .collect()
    }
    match value {
        RegValue::Str(s) => (REG_SZ, utf16_bytes(s)),
        RegValue::ExpandStr(s) => (REG_EXPAND_SZ, utf16_bytes(s)),
        RegValue::MultiStr(v) => {
            let mut bytes: Vec<u8> = v.iter().flat_map(|s| utf16_bytes(s)).collect();
            bytes.extend_from_slice(&0u16.to_le_bytes()); // final terminator
            (REG_MULTI_SZ, bytes)
        }
        RegValue::Dword(n) => (REG_DWORD, n.to_le_bytes().to_vec()),
        RegValue::Qword(n) => (REG_QWORD, n.to_le_bytes().to_vec()),
        RegValue::Binary(b) => (REG_BINARY, b.clone()),
        RegValue::Other(_) => (REG_BINARY, Vec::new()),
    }
}

/// The four places installed programs register themselves.
pub const UNINSTALL_ROOTS: &[(&str, RegView)] = &[
    (
        r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
        RegView::Bits64,
    ),
    (
        r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
        RegView::Bits32,
    ),
];

/// Convenience for the enumerate-subkeys access mask.
pub fn enumerate_access() -> REG_SAM_FLAGS {
    KEY_QUERY_VALUE | KEY_ENUMERATE_SUB_KEYS | KEY_SET_VALUE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hive_prefixes_parse_in_both_spellings() {
        assert!(split_hive(r"HKLM\SOFTWARE\X").is_some());
        assert!(split_hive(r"HKEY_LOCAL_MACHINE\SOFTWARE\X").is_some());
        assert!(split_hive(r"hkcu\Software").is_some());
        assert!(split_hive(r"NOT_A_HIVE\X").is_none());
    }

    #[test]
    fn hive_split_returns_the_remainder() {
        let (_, rest) = split_hive(r"HKLM\SOFTWARE\Microsoft").unwrap();
        assert_eq!(rest, r"SOFTWARE\Microsoft");
    }

    #[test]
    fn string_values_round_trip_through_encoding() {
        let (kind, bytes) = encode_value(&RegValue::Str("Hello".into()));
        assert_eq!(kind, REG_SZ);
        assert_eq!(decode_value(kind, &bytes), RegValue::Str("Hello".into()));
    }

    #[test]
    fn dword_values_round_trip() {
        let (kind, bytes) = encode_value(&RegValue::Dword(0xDEAD_BEEF));
        assert_eq!(decode_value(kind, &bytes), RegValue::Dword(0xDEAD_BEEF));
    }

    #[test]
    fn multi_string_values_round_trip() {
        let v = RegValue::MultiStr(vec!["a".into(), "b".into()]);
        let (kind, bytes) = encode_value(&v);
        assert_eq!(decode_value(kind, &bytes), v);
    }

    #[test]
    fn dword_reads_as_a_string_too() {
        assert_eq!(RegValue::Dword(42).as_string().as_deref(), Some("42"));
        assert_eq!(RegValue::Str("42".into()).as_u32(), Some(42));
    }

    #[test]
    fn both_registry_views_are_scanned() {
        // Missing the 32-bit view hides roughly half the installed programs
        // on a typical machine, so pin it down.
        assert!(UNINSTALL_ROOTS.iter().any(|(_, v)| *v == RegView::Bits64));
        assert!(UNINSTALL_ROOTS.iter().any(|(_, v)| *v == RegView::Bits32));
    }
}
