use common::cleanup_entry;
use uuid::Uuid;
use win_ctx::*;

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

    assert!(entry.children().unwrap().is_empty());
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

    assert_eq!(
        parent.children().unwrap().len(),
        2,
        "Parent should have two children"
    );
    assert!(
        parent.child(&child_1_id).unwrap().is_some(),
        "Parent should have child 1"
    );
    assert!(
        parent.child(&child_2_id).unwrap().is_some(),
        "Parent should have child 2"
    );
    assert!(child_1.parent().is_some(), "Child 1 should have parent");
    assert!(child_2.parent().is_some(), "Child 2 should have parent");
    assert_eq!(
        child_1.parent().unwrap().path(),
        child_2.parent().unwrap().path(),
        "Children do not have same parent"
    );
    cleanup_entry(parent);
}

#[test]
fn orphan_basic() {
    let parent_id = Uuid::new_v4().to_string();
    let child_id = Uuid::new_v4().to_string();
    let parent = CtxEntry::new(&parent_id, &ActivationType::Folder).unwrap();
    let child = parent.new_child(&child_id).unwrap();

    parent.delete().expect("Failed to delete parent");
    assert!(child.parent().is_none(), "Child has parent after deletion");
}

#[test]
fn orphan_with_error_value() {
    let parent_id = Uuid::new_v4().to_string();
    let child_id = Uuid::new_v4().to_string();
    let parent = CtxEntry::new(&parent_id, &ActivationType::Folder).unwrap();
    let mut child = parent.new_child(&child_id).unwrap();

    child
        .set_icon(Some("test icon"))
        .expect("Failed to set child icon");
    parent.delete().expect("Failed to delete parent");
    child
        .icon()
        .expect_err("Should not be able to get child icon after orphaned");
}
