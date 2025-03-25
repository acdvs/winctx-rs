use common::cleanup_entry;
use uuid::Uuid;
use winctx::*;

mod common;

#[test]
fn root_has_no_parent() {
    let id = Uuid::new_v4().to_string();
    let entry = CtxEntry::new(&id, &ActivationType::Folder).unwrap();

    assert!(entry.parent().is_none());
    cleanup_entry(entry);
}

#[test]
fn new_entry_has_no_children() {
    let id = Uuid::new_v4().to_string();
    let entry = CtxEntry::new(&id, &ActivationType::Folder).unwrap();

    assert!(entry.children().is_empty());
    cleanup_entry(entry);
}

#[test]
fn parent_with_children() {
    let parent_id = Uuid::new_v4().to_string();
    let child_1_id = Uuid::new_v4().to_string();
    let child_2_id = Uuid::new_v4().to_string();
    let parent = CtxEntry::new(&parent_id, &ActivationType::Folder).unwrap();
    let child_1 = parent.new_child(&child_1_id).unwrap();
    let child_2 = parent.new_child(&child_2_id).unwrap();

    assert!(
        parent.child(&child_1_id).is_some(),
        "Parent should have child 1"
    );
    assert!(
        parent.child(&child_2_id).is_some(),
        "Parent should have child 2"
    );
    assert!(child_1.parent().is_some(), "Child 1 should have parent");
    assert!(child_2.parent().is_some(), "Child 2 should have parent");
    assert_eq!(
        child_1.parent().unwrap().get_full_path(),
        child_2.parent().unwrap().get_full_path(),
        "Children do not have same parent"
    );
    cleanup_entry(parent);
}

#[test]
fn child_with_deleted_parent() {
    let parent_id = Uuid::new_v4().to_string();
    let child_id = Uuid::new_v4().to_string();
    let parent = CtxEntry::new(&parent_id, &ActivationType::Folder).unwrap();
    let child = parent.new_child(&child_id).unwrap();

    parent.delete().expect("Failed to delete parent");
    assert!(child.parent().is_none(), "Child has parent after deletion");
}
