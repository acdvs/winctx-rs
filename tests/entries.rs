use common::*;
use std::io::ErrorKind;
use uuid::Uuid;
use win_ctx::*;
use winreg::{RegKey, enums::*};

mod common;

const HKCR: RegKey = RegKey::predef(HKEY_CLASSES_ROOT);

#[test]
fn get_entry() {
    let id = Uuid::new_v4().to_string();
    let new_entry = CtxEntry::new(&id, &ActivationType::Folder).unwrap();
    let entry = CtxEntry::get(&[&id], &ActivationType::Folder).unwrap();
    let name = entry.name().unwrap();

    assert_eq!(name, id);
    cleanup_entry(new_entry);
}

#[test]
fn get_all_entries() {
    let file_entries = CtxEntry::get_all_of_type(&ActivationType::File("*".to_string()));
    let rs_ext_entries = CtxEntry::get_all_of_type(&ActivationType::File(".rs".to_string()));
    let folder_entries = CtxEntry::get_all_of_type(&ActivationType::Folder);
    let bg_entries = CtxEntry::get_all_of_type(&ActivationType::Background);

    // No nice way to test these independently of all systems.
    assert_eq!(file_entries.len(), 3);
    assert_eq!(rs_ext_entries.len(), 0);
    assert_eq!(folder_entries.len(), 4);
    assert_eq!(bg_entries.len(), 3);
}

#[test]
fn get_missing_entry() {
    let id = Uuid::new_v4().to_string();
    let entry = CtxEntry::get(&[id], &ActivationType::Background);

    assert!(entry.is_none());
}

#[test]
fn create_and_get_entry() {
    let id = Uuid::new_v4().to_string();
    let new_entry = CtxEntry::new(&id, &ActivationType::Folder).unwrap();
    let get_entry = CtxEntry::get(&[id], &ActivationType::Folder).unwrap();

    assert_eq!(&new_entry.path, &get_entry.path);
    cleanup_entry(new_entry);
}

#[test]
fn basic_entry_on_all_files() {
    let id = Uuid::new_v4().to_string();
    let entry = CtxEntry::new(&id, &ActivationType::File("*".to_string())).unwrap();
    let key = HKCR.open_subkey(format!("*\\shell\\{id}")).unwrap();

    assert_reg_command!(false, key);
    assert_reg_value!(false, key, "Icon");
    assert_reg_value!(false, key, "Position");
    assert_reg_value!(false, key, "Extended");
    cleanup_entry(entry);
}

#[test]
fn basic_entry_on_extension() {
    let id = Uuid::new_v4().to_string();
    let entry = CtxEntry::new(&id, &ActivationType::File(".rs".to_string())).unwrap();
    let key = HKCR.open_subkey(format!(".rs\\shell\\{id}")).unwrap();

    assert_reg_command!(false, key);
    assert_reg_value!(false, key, "Icon");
    assert_reg_value!(false, key, "Position");
    assert_reg_value!(false, key, "Extended");
    cleanup_entry(entry);
}

#[test]
fn basic_entry_on_folder() {
    let id = Uuid::new_v4().to_string();
    let entry = CtxEntry::new(&id, &ActivationType::Folder).unwrap();
    let key = HKCR.open_subkey(format!("Directory\\shell\\{id}")).unwrap();

    assert_reg_command!(false, key);
    assert_reg_value!(false, key, "Icon");
    assert_reg_value!(false, key, "Position");
    assert_reg_value!(false, key, "Extended");
    cleanup_entry(entry);
}

#[test]
fn basic_entry_on_background() {
    let id = Uuid::new_v4().to_string();
    let entry = CtxEntry::new(&id, &ActivationType::Background).unwrap();
    let key = HKCR
        .open_subkey(format!("Directory\\Background\\shell\\{id}"))
        .unwrap();

    assert_reg_command!(false, key);
    assert_reg_value!(false, key, "Icon");
    assert_reg_value!(false, key, "Position");
    assert_reg_value!(false, key, "Extended");
    cleanup_entry(entry);
}

#[test]
fn entry_with_options() {
    let id = Uuid::new_v4().to_string();
    let test_str = "test string";
    let entry = CtxEntry::new_with_options(
        &id,
        &ActivationType::Background,
        &EntryOptions {
            command: Some(test_str.to_string()),
            icon: Some(test_str.to_string()),
            position: Some(MenuPosition::Top),
            separator: Some(Separator::After),
            extended: true,
        },
    )
    .unwrap();
    let key = HKCR
        .open_subkey(format!("Directory\\Background\\shell\\{id}"))
        .unwrap();

    assert_reg_command!(true, key, test_str);
    assert_reg_value!(true, key, "Icon", test_str);
    assert_reg_value!(true, key, "Position", "Top");
    assert_reg_value!(true, key, "Extended", "");
    cleanup_entry(entry);
}

#[test]
fn get_and_set_command() {
    let id = Uuid::new_v4().to_string();
    let command = "test command";
    let mut entry = CtxEntry::new(&id, &ActivationType::Folder).unwrap();
    let key = HKCR.open_subkey(format!("Directory\\shell\\{id}")).unwrap();

    assert_reg_command!(false, key);
    entry
        .set_command(Some(command))
        .expect("Failed to set command");
    assert_reg_command!(true, key, command);
    assert_eq!(entry.command().unwrap().unwrap(), command);
    cleanup_entry(entry);
}

#[test]
fn get_and_set_icon() {
    let id = Uuid::new_v4().to_string();
    let icon = "test icon";
    let mut entry = CtxEntry::new(&id, &ActivationType::Folder).unwrap();
    let key = HKCR.open_subkey(format!("Directory\\shell\\{id}")).unwrap();

    assert_reg_value!(false, key, "Icon");
    entry.set_icon(Some(icon)).expect("Failed to set icon");
    assert_reg_value!(true, key, "Icon", icon);
    assert_eq!(entry.icon().unwrap().unwrap(), icon);
    cleanup_entry(entry);
}

#[test]
fn get_and_set_position() {
    let id = Uuid::new_v4().to_string();
    let position = MenuPosition::Bottom;
    let mut entry = CtxEntry::new(&id, &ActivationType::Folder).unwrap();
    let key = HKCR.open_subkey(format!("Directory\\shell\\{id}")).unwrap();

    assert_reg_value!(false, key, "Position");
    entry
        .set_position(Some(position))
        .expect("Failed to set position");
    assert_reg_value!(true, key, "Position", "Bottom");
    cleanup_entry(entry);
}

#[test]
fn get_and_set_separator() {
    let id = Uuid::new_v4().to_string();
    let mut entry = CtxEntry::new(&id, &ActivationType::Folder).unwrap();
    let key = HKCR.open_subkey(format!("Directory\\shell\\{id}")).unwrap();

    assert_reg_value!(false, key, "SeparatorBefore");
    assert_reg_value!(false, key, "SeparatorAfter");
    entry
        .set_separator(Some(Separator::Both))
        .expect("Failed to set separator");
    assert_reg_value!(true, key, "SeparatorBefore", "");
    assert_reg_value!(true, key, "SeparatorAfter", "");
    cleanup_entry(entry);
}

#[test]
fn get_and_set_extended() {
    let id = Uuid::new_v4().to_string();
    let mut entry = CtxEntry::new(&id, &ActivationType::Folder).unwrap();
    let key = HKCR.open_subkey(format!("Directory\\shell\\{id}")).unwrap();

    assert_reg_value!(false, key, "Extended");
    entry.set_extended(true).expect("Failed to set extended");
    assert_reg_value!(true, key, "Extended", "");
    assert_eq!(entry.extended().unwrap(), true);
    cleanup_entry(entry);
}

#[test]
fn rename_entry() {
    let old_id = Uuid::new_v4().to_string();
    let new_id = Uuid::new_v4().to_string();
    let mut entry = CtxEntry::new(&old_id, &ActivationType::Folder).unwrap();

    HKCR.open_subkey(format!("Directory\\shell\\{}", old_id))
        .expect("Initial entry does not exist");
    entry.rename(&new_id).expect("Failed to rename entry");
    HKCR.open_subkey(format!("Directory\\shell\\{}", new_id))
        .expect("Renamed entry does not exist");

    entry
        .rename("")
        .expect_err("Blank name should not be allowed");

    cleanup_entry(entry);
}

#[test]
fn delete_entry() {
    let id = Uuid::new_v4().to_string();
    let entry = CtxEntry::new(&id, &ActivationType::Folder).unwrap();

    entry.delete().expect("Failed to delete entry");
    HKCR.open_subkey(format!("Directory\\shell\\{id}"))
        .expect_err("Found key after deletion");
}
