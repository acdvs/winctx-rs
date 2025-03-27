use crate::path::CTX_MENU_PATH;
use std::io::{self, ErrorKind};
use winreg::{RegKey, enums::HKEY_CURRENT_USER};

const HKCU: RegKey = RegKey::predef(HKEY_CURRENT_USER);

/// Enable or disable the pre-Windows 11 context menu.
/// You must restart explorer.exe for changes to take effect.
///
/// # Examples
///
/// ```no_run
/// win_ctx::toggle_classic_menu(true)?;
/// ```
pub fn toggle_classic_menu(enable: bool) -> io::Result<()> {
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
    fn disable_enable_classic_menu() {
        toggle_classic_menu(false).expect("Failed to disable classic menu");
        toggle_classic_menu(false).expect("Duplicate menu disable call should be ok");
        HKCU.open_subkey(CTX_MENU_PATH)
            .expect_err("Classic menu key should not exist");

        toggle_classic_menu(true).expect("Failed to enable classic menu");
        toggle_classic_menu(true).expect("Duplicate menu enable call should be ok");
        HKCU.open_subkey(CTX_MENU_PATH)
            .expect("Classic menu key should exist");
    }
}
