use super::path::*;
use std::collections::HashMap;
use std::io::{self, ErrorKind};
use winreg::{RegKey, enums::*};

const HKCR: RegKey = RegKey::predef(HKEY_CLASSES_ROOT);

/// Entry activation type
#[derive(Clone)]
pub enum ActivationType {
    /// Entry activation on file right click. Must be an extension (e.g., `.rs`) or `*` for any file type
    File(String),
    /// Entry activation on folder right click.
    Folder,
    /// Entry activation on directory background right click.
    Background,
}

/// Entry position in the context menu
#[derive(Clone)]
pub enum MenuPosition {
    Top,
    Bottom,
}

/// Context menu separator
#[derive(Clone)]
pub enum Separator {
    Before,
    After,
    Both,
}

pub struct CtxEntry {
    /// The path to the entry as a list of entry names
    pub path: Vec<String>,
    pub entry_type: ActivationType,
}

/// Options for further customizing an entry
#[derive(Clone)]
pub struct EntryOptions {
    /// Command to run when the entry is selected
    pub command: Option<String>,
    /// Icon to display beside the entry
    pub icon: Option<String>,
    /// Entry position in the context menu
    pub position: Option<MenuPosition>,
    /// Separators to include around the entry
    pub separator: Option<Separator>,
    /// Whether the entry should only appear with Shift+RClick
    pub extended: bool,
}

impl CtxEntry {
    /// Gets an existing entry at the given name path. The last name
    /// corresponds to the returned entry.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let name_path = &["Root entry", "Sub entry", "Sub sub entry"];
    /// let entry = CtxEntry::get(name_path, &ActivationType::Folder)?;
    /// ``````
    pub fn get<N: AsRef<str>>(name_path: &[N], entry_type: &ActivationType) -> Option<CtxEntry> {
        if name_path.len() == 0 {
            return None;
        }

        let mut str_path = get_base_path(&entry_type);

        for entry_name in name_path.iter().map(|x| x.as_ref()) {
            str_path.push_str(&format!("\\shell\\{entry_name}"));
        }

        let key = get_key(&str_path);

        if key
            .as_ref()
            .err()
            .map_or(false, |e| e.kind() == ErrorKind::NotFound)
        {
            return None;
        }

        Some(CtxEntry {
            path: name_path.iter().map(|x| x.as_ref().to_string()).collect(),
            entry_type: entry_type.clone(),
        })
    }

    pub fn get_all_of_type(entry_type: &ActivationType) -> HashMap<String, CtxEntry> {
        let mut entries = HashMap::new();

        let base_path = get_base_path(&entry_type);
        let shell_path = format!("{base_path}\\shell");
        let shell_key = match get_key(&shell_path) {
            Ok(key) => key,
            Err(_) => return entries,
        };

        for entry_name in shell_key.enum_keys().map(|x| x.unwrap()) {
            match CtxEntry::get(&[entry_name.clone()], entry_type) {
                Some(entry) => {
                    entries.insert(entry_name, entry);
                }
                None => (),
            };
        }

        entries
    }

    fn create(
        name_path: &[String],
        entry_type: &ActivationType,
        opts: &EntryOptions,
    ) -> io::Result<CtxEntry> {
        let path_str = get_full_path(entry_type, name_path);
        let (_, disp) = HKCR.create_subkey(path_str)?;

        if disp == REG_OPENED_EXISTING_KEY {
            return Err(io::Error::from(ErrorKind::AlreadyExists));
        }

        let mut entry = CtxEntry {
            path: name_path.to_vec(),
            entry_type: entry_type.clone(),
        };

        entry.set_command(opts.command.as_deref())?;
        entry.set_icon(opts.icon.as_deref())?;
        entry.set_position(opts.position.clone())?;
        entry.set_extended(opts.extended)?;

        Ok(entry)
    }

    /// Creates a new top-level entry under the given `entry_type`.
    /// The resulting entry will appear in the context menu but will do
    /// nothing until modified.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let mut entry = CtxEntry::new("Basic entry", ActivationType::Background)?;
    /// ```
    pub fn new(name: &str, entry_type: &ActivationType) -> io::Result<CtxEntry> {
        CtxEntry::new_with_options(
            name,
            entry_type,
            &EntryOptions {
                command: None,
                icon: None,
                position: None,
                separator: None,
                extended: false,
            },
        )
    }

    /// Creates a new top-level entry under the given `entry_type`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let entry = CtxEntry::new(
    ///     "Open in terminal",
    ///     &ActivationType::Folder,
    ///     &EntryOptions {
    ///         // This command opens the target directory in cmd.
    ///         command: Some("cmd /s /k pushd \"%V\""),
    ///         icon: Some("C:\\Windows\\System32\\cmd.exe"),
    ///         position: None,
    ///         extended: false,
    ///     }
    /// )?;
    /// ```
    pub fn new_with_options(
        name: &str,
        entry_type: &ActivationType,
        opts: &EntryOptions,
    ) -> io::Result<CtxEntry> {
        let name_path = [name.to_string()];
        CtxEntry::create(&name_path, entry_type, opts)
    }

    /// Deletes the entry and any children.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let entry = CtxEntry::new("Basic entry", ActivationType::Background)?;
    /// entry.delete()?;
    /// ```
    pub fn delete(self) -> io::Result<()> {
        HKCR.delete_subkey_all(self.path())
    }

    /// Gets the entry's current name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let entry = CtxEntry::new("Basic entry", ActivationType::Background)?;
    /// let name = entry.name()?;
    /// ```
    pub fn name(&self) -> io::Result<String> {
        let _ = self.key()?;
        Ok(self.path.last().unwrap().to_owned())
    }

    /// Renames the entry.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let mut entry = CtxEntry::new("Basic entry", ActivationType::Background)?;
    /// entry.rename("Renamed entry")?;
    /// ```
    pub fn rename(&mut self, new_name: &str) -> io::Result<()> {
        if new_name.len() == 0 {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "Name cannot be empty",
            ));
        }

        let old_name = self.name()?;

        let parent_name_path = &self.path[..self.path.len() - 1];
        let parent_path_str = get_full_path(&self.entry_type, parent_name_path);
        let parent_key = HKCR.open_subkey(parent_path_str)?;
        let res = parent_key.rename_subkey(old_name, new_name);

        let path_len = self.path.len();
        self.path[path_len - 1] = new_name.to_string();

        res
    }

    /// Gets the entry's command, if any.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let entry = CtxEntry::new("Basic entry", ActivationType::Background)?;
    /// let command = entry.command()?;
    /// ```
    pub fn command(&self) -> io::Result<Option<String>> {
        let path = format!(r"{}\command", self.path());
        let key = get_key(&path)?;
        Ok(key.get_value::<String, _>("").ok())
    }

    /// Sets the entry's command.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let mut entry = CtxEntry::new("Basic entry", ActivationType::Folder)?;
    /// // This command opens the target directory in Powershell.
    /// entry.set_command(Some("powershell.exe -noexit -command Set-Location -literalPath '%V'"))?;
    /// ```
    pub fn set_command(&mut self, command: Option<&str>) -> io::Result<()> {
        let key = self.key()?;
        match command {
            Some(c) => {
                let (command_key, _) = key.create_subkey("command")?;
                command_key.set_value("", &c)
            }
            None => match key.delete_subkey("command") {
                Err(e) if e.kind() == ErrorKind::NotFound => Ok(()),
                Err(e) => Err(e),
                Ok(_) => Ok(()),
            },
        }
    }

    /// Gets the entry's icon, if any.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let entry = CtxEntry::new("Basic entry", ActivationType::Background)?;
    /// let icon = entry.icon()?;
    /// ```
    pub fn icon(&self) -> io::Result<Option<String>> {
        let key = self.key()?;
        Ok(key.get_value::<String, _>("Icon").ok())
    }

    /// Sets the entry's icon.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let mut entry = CtxEntry::new("Basic entry", ActivationType::Background)?;
    /// entry.set_icon(Some("C:\\Windows\\System32\\control.exe"))?;
    /// ```
    pub fn set_icon(&mut self, icon: Option<&str>) -> io::Result<()> {
        let key = self.key()?;
        match icon {
            Some(icon) => key.set_value("Icon", &icon),
            None => self.safe_delete_value("Icon"),
        }
    }

    /// Gets the entry's position, if any.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let entry = CtxEntry::new("Basic entry", ActivationType::Background)?;
    /// let position = entry.position()?;
    /// ```
    pub fn position(&self) -> io::Result<Option<MenuPosition>> {
        let key = self.key()?;
        let val = match key.get_value::<String, _>("Position") {
            Ok(v) if v == "Top" => Some(MenuPosition::Top),
            Ok(v) if v == "Bottom" => Some(MenuPosition::Bottom),
            _ => None,
        };

        Ok(val)
    }

    /// Sets the entry's menu position. By default, new root entries are
    /// positioned at the top. Does not affect child entries.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let mut entry = CtxEntry::new("Basic entry", ActivationType::Background)?;
    /// entry.set_position(Some(MenuPosition::Bottom))?;
    /// ```
    pub fn set_position(&mut self, position: Option<MenuPosition>) -> io::Result<()> {
        if position.is_none() {
            return self.safe_delete_value("Position");
        }

        let position_str = match position {
            Some(MenuPosition::Top) => "Top",
            Some(MenuPosition::Bottom) => "Bottom",
            None => "",
        };

        self.key()?.set_value("Position", &position_str)
    }

    /// Gets whether the entry appears with Shift+RClick.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let entry = CtxEntry::new("Basic entry", ActivationType::Background)?;
    /// let is_extended = entry.extended()?;
    /// ```
    pub fn extended(&self) -> io::Result<bool> {
        let key = self.key()?;
        Ok(key.get_value::<String, _>("Extended").ok().is_some())
    }

    /// Sets whether the entry should only appear with Shift+RClick.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let mut entry = CtxEntry::new("Basic entry", ActivationType::Background)?;
    /// entry.set_extended(true)?;
    /// ```
    pub fn set_extended(&mut self, extended: bool) -> io::Result<()> {
        if extended {
            self.key()?.set_value("Extended", &"")
        } else {
            self.safe_delete_value("Extended")
        }
    }

    /// Gets the entry's separator(s), if any.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let entry = CtxEntry::new("Basic entry", ActivationType::Background)?;
    /// let separator = entry.separator()?;
    /// ```
    pub fn separator(&self) -> io::Result<Option<Separator>> {
        let key = self.key()?;
        let sep_before = key.get_value::<String, _>("SeparatorBefore");
        let sep_after = key.get_value::<String, _>("SeparatorAfter");

        Ok(match (sep_before, sep_after) {
            (Ok(_), Ok(_)) => Some(Separator::Both),
            (Ok(_), Err(_)) => Some(Separator::Before),
            (Err(_), Ok(_)) => Some(Separator::After),
            _ => None,
        })
    }

    /// Sets the entry's separator(s).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let mut entry = CtxEntry::new("Basic entry", ActivationType::Background)?;
    /// entry.set_separator(Some(Separator::After))?;
    /// ```
    pub fn set_separator(&mut self, separator: Option<Separator>) -> io::Result<()> {
        let key = self.key()?;
        match separator {
            Some(Separator::Before) => {
                key.set_value("SeparatorBefore", &"")?;
                self.safe_delete_value("SeparatorAfter")?;
                Ok(())
            }
            Some(Separator::After) => {
                key.set_value("SeparatorAfter", &"")?;
                self.safe_delete_value("SeparatorBefore")?;
                Ok(())
            }
            Some(Separator::Both) => {
                key.set_value("SeparatorBefore", &"")?;
                key.set_value("SeparatorAfter", &"")?;
                Ok(())
            }
            None => {
                self.safe_delete_value("SeparatorBefore")?;
                self.safe_delete_value("SeparatorAfter")?;
                Ok(())
            }
        }
    }

    /// Gets the entry's parent, if any.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let entry = CtxEntry::new("Basic entry", ActivationType::Background)?;
    /// let child = entry.new_child("Basic child entry")?;
    /// let parent = child.parent()?;
    /// assert_eq!(entry.name().unwrap(), parent.name().unwrap());
    /// ```
    pub fn parent(&self) -> Option<CtxEntry> {
        if self.path.len() <= 1 {
            return None;
        }

        let parent_path = &self.path[..self.path.len() - 1];
        CtxEntry::get(parent_path, &self.entry_type)
    }

    /// Gets one of the entry's children, if any.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let entry = CtxEntry::new("Basic entry", ActivationType::Background)?;
    /// let created_child = entry.new_child("Basic child entry")?;
    /// let retrieved_child = entry.child("Basic child entry")?;
    /// assert_eq!(created_child.name().unwrap(), retrieved_child.name().unwrap());
    /// ```
    pub fn child(&self, name: &str) -> io::Result<Option<CtxEntry>> {
        let mut name_path = self.path.clone();
        name_path.push(name.to_string());
        let path_str = get_full_path(&self.entry_type, &name_path);

        match get_key(&path_str) {
            Ok(_) => Ok(Some(CtxEntry {
                path: name_path,
                entry_type: self.entry_type.clone(),
            })),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Gets the entry's children, if any.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let entry = CtxEntry::new("Basic entry", ActivationType::Background)?;
    /// let child_1 = entry.new_child("Child 1")?;
    /// let child_2 = entry.new_child("Child 2")?;
    /// let children = entry.children()?;
    /// ```
    pub fn children(&self) -> io::Result<Vec<CtxEntry>> {
        let key = self.key()?;
        let mut children = Vec::new();

        for name in key.enum_keys().map(|x| x.unwrap()) {
            let child = self.child(&name).unwrap().unwrap();
            children.push(child);
        }

        Ok(children)
    }

    /// Creates a new child entry under the entry. The resulting entry
    /// will appear in the context menu but will do nothing until modified.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let entry = CtxEntry::new("Basic entry", ActivationType::Background)?;
    /// let child = entry.new_child("Basic child entry")?;
    /// ```
    pub fn new_child(&self, name: &str) -> io::Result<CtxEntry> {
        self.new_child_with_options(
            name,
            &EntryOptions {
                command: None,
                icon: None,
                position: None,
                separator: None,
                extended: false,
            },
        )
    }

    /// Creates a new child entry under the entry.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let entry = CtxEntry::new("Basic entry", ActivationType::Background)?;
    /// let child = entry.new_child_with_options(
    ///     "Basic child entry",
    ///     &EntryOptions {
    ///         // This command opens the target directory in cmd.
    ///         command: Some("cmd /s /k pushd \"%V\""),
    ///         icon: Some("C:\\Windows\\System32\\cmd.exe"),
    ///         position: None,
    ///         extended: false,
    ///     }
    /// )?;
    /// ```
    pub fn new_child_with_options(&self, name: &str, opts: &EntryOptions) -> io::Result<CtxEntry> {
        let key = self.key()?;
        key.set_value("Subcommands", &"")?;

        let mut path = self.path.clone();
        path.push(name.to_string());

        CtxEntry::create(path.as_slice(), &self.entry_type, &opts)
    }

    /// Gets the full path to the entry's registry key.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let entry = CtxEntry::new("Basic entry", ActivationType::Background)?;
    /// let path = entry.path();
    /// ```
    pub fn path(&self) -> String {
        get_full_path(&self.entry_type, &self.path)
    }

    // Shortcut to get the entry's registry key.
    // Should be checked before every operation.
    fn key(&self) -> io::Result<RegKey> {
        get_key(&self.path())
    }

    // Delete value without erroring if nonexistent.
    fn safe_delete_value(&self, value: &str) -> io::Result<()> {
        let key = self.key()?;
        match key.delete_value(value) {
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e),
            Ok(_) => Ok(()),
        }
    }
}

fn get_key(path: &str) -> io::Result<RegKey> {
    match HKCR.open_subkey_with_flags(path, KEY_ALL_ACCESS) {
        Err(e) if e.kind() == ErrorKind::NotFound => Err(io::Error::new(
            ErrorKind::NotFound,
            "Registry key does not exist",
        )),
        Err(e) => Err(e),
        Ok(key) => Ok(key),
    }
}
