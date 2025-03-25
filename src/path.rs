use super::ActivationType;

/// Exposed for testing purposes only.
pub const CTX_MENU_PATH: &str = "Software\\Classes\\CLSID\\{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}";

/// Exposed for testing purposes only.
pub fn get_base_path(entry_type: &ActivationType) -> String {
    match entry_type {
        ActivationType::File(ext) => ext.to_string(),
        ActivationType::Folder => "Directory".to_string(),
        ActivationType::Background => "Directory\\Background".to_string(),
    }
}

/// Exposed for testing purposes only.
pub fn get_full_path<N: AsRef<str>>(entry_type: &ActivationType, name_path: &[N]) -> String {
    let mut path = get_base_path(entry_type);

    if name_path.len() == 0 {
        path.push_str("\\shell");
    }

    for entry_name in name_path.iter().map(|x| x.as_ref()) {
        path = format!("{path}\\shell\\{entry_name}");
    }

    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_name_path() {
        let path = get_full_path(&ActivationType::Folder, &Vec::<&str>::new());
        assert_eq!(path, "Directory\\shell");
    }

    #[test]
    fn filled_name_path() {
        let path = get_full_path(&ActivationType::Folder, &["1", "2", "3"]);
        assert_eq!(path, "Directory\\shell\\1\\shell\\2\\shell\\3");
    }
}
