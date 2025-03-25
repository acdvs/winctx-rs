use winctx::CtxEntry;
use winreg::{RegKey, enums::HKEY_CLASSES_ROOT};

pub fn cleanup_entry(entry: CtxEntry) {
    let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);
    let path = entry.get_full_path();
    hkcr.delete_subkey_all(&path).expect("Key cleanup failed");
}

#[macro_export]
macro_rules! assert_reg_command {
    (false, $k:expr) => {
        $k.open_subkey("command")
            .expect_err("Entry should not have a command");
    };
    (true, $k:expr, $v:expr) => {
        let key = $k
            .open_subkey("command")
            .expect("Entry should have a command");
        assert!(
            key.get_value::<String, _>("").is_ok_and(|v| v == $v),
            "Entry has incorrect command value"
        );
    };
}

#[macro_export]
macro_rules! assert_reg_value {
    (true, $k:expr, $v:literal, $val:expr) => {
        let v_str = $v;
        let res = $k.get_value::<String, _>($v);
        assert!(res.is_ok(), "Entry has missing {v_str} value");
        assert!(res.unwrap() == $val, "Entry has incorrect {v_str} value");
    };
    (false, $k:expr, $v:literal) => {
        let v_str = $v;
        assert!(
            $k.get_value::<String, _>("$v")
                .is_err_and(|e| e.kind() == ErrorKind::NotFound),
            "Entry has {v_str} value but should not"
        );
    };
}
