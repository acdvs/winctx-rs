use common::*;
use std::io::ErrorKind;
use uuid::Uuid;
use win_ctx::*;
use winreg::{RegKey, enums::*};

mod common;

const HKCR: RegKey = RegKey::predef(HKEY_CLASSES_ROOT);

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
    assert_eq!(entry.command().unwrap(), command);
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
    assert_eq!(entry.icon().unwrap(), icon);
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
fn get_and_set_extended() {
    let id = Uuid::new_v4().to_string();
    let mut entry = CtxEntry::new(&id, &ActivationType::Folder).unwrap();
    let key = HKCR.open_subkey(format!("Directory\\shell\\{id}")).unwrap();

    assert_reg_value!(false, key, "Extended");
    entry.set_extended(true).expect("Failed to set extended");
    assert_reg_value!(true, key, "Extended", "");
    assert_eq!(entry.extended(), true);
    cleanup_entry(entry);
}

#[test]
fn rename_entry() {
    let old_id = Uuid::new_v4().to_string();
    let new_id = Uuid::new_v4().to_string();
    let mut entry = CtxEntry::new(&old_id, &ActivationType::Folder).unwrap();

    let old_key = HKCR.open_subkey(format!("Directory\\shell\\{}", old_id));
    assert!(old_key.is_ok(), "Initial entry does not exist");

    entry.rename(&new_id).expect("Failed to rename entry");
    let new_key = HKCR.open_subkey(format!("Directory\\shell\\{}", new_id));
    assert!(new_key.is_ok(), "Renamed entry does not exist");
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
