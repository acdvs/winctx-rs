use crate::path::CTX_MENU_PATH;
use std::io::{self, ErrorKind};
use winreg::{RegKey, enums::HKEY_CURRENT_USER};

const HKCU: RegKey = RegKey::predef(HKEY_CURRENT_USER);

/// Enable or disable the old Windows context menu.
/// You must restart explorer.exe for changes to take effect.
///
/// # Examples
///
/// ```no_run
/// winctx::toggle_old_menu(true);
/// ```
pub fn toggle_old_menu(enable: bool) -> io::Result<()> {
    if enable {
        let path = format!("{}\\InprocServer32\\", CTX_MENU_PATH);
        let (key, _) = HKCU.create_subkey(path).unwrap();
        key.set_value("", &"")
    } else {
        match HKCU.delete_subkey_all(CTX_MENU_PATH) {
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e),
            Ok(_) => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enable_old_menu() {
        toggle_old_menu(true).expect("Failed to enable old context menu");
        // Duplicate calls shouldn't panic
        toggle_old_menu(true).expect("Failed to enable old context menu");
        HKCU.open_subkey(CTX_MENU_PATH)
            .expect("Old context menu key should exist");
    }

    #[test]
    fn disable_old_menu() {
        toggle_old_menu(false).expect("Failed to disable old context menu");
        // Duplicate calls shouldn't panic
        toggle_old_menu(false).expect("Failed to disable old context menu");
        HKCU.open_subkey(CTX_MENU_PATH)
            .expect_err("Old context menu key should not exist");
    }
}
