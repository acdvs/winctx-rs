//! A Rust library for managing Windows context menu entries.
//!
//! ## Installation
//!
//! ```sh
//! cargo add win-ctx
//! ```
//!
//! ## Features
//!
//! - Create and edit context menu entries and sub-entries
//! - Toggle the pre-Windows 11 context menu
//!
//! ## Basic example
//!
//! The following code creates a top-level context menu entry that appears on
//! right-clicked folders and opens the target folder in the terminal.
//!
//! ```no_run
//! use win_ctx::*;
//!
//! CtxEntry::new_with_options(
//!     "Open in terminal",
//!     &ActivationType::Folder,
//!     &EntryOptions {
//!         command: Some("cmd /s /k pushd \"%V\""),
//!         icon: Some("C:\\Windows\\System32\\cmd.exe"),
//!         position: None,
//!         extended: false,
//!     }
//! )?;
//! ```
//!
//! ## Advanced example
//!
//! The following code creates a context menu entry with child entries that each
//! open the target folder in the selected program.
//!
//! To reduce line count, the more basic non-options functions can be used,
//! and individual values are then set on the resulting entries.
//!
//! ```no_run
//! use win_ctx::{CtxEntry, ActivationType};
//!
//! let mut parent = CtxEntry::new("Open directory in", &ActivationType::Background)?;
//! parent.set_extended(true);
//!
//! let mut child_1 = parent.new_child("Terminal")?;
//! child_1.set_command(Some("cmd /s /k pushd \"%V\""))?;
//! child_1.set_icon(Some("C:\\Windows\\System32\\cmd.exe"))?;
//!
//! let mut child_2 = parent.new_child("Powershell")?;
//! child_2.set_command(Some("powershell -noexit -command Set-Location -literalPath '%V'"))?;
//! child_2.set_icon(Some("C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe"))?;
//! ```
//!
//! ## Errors
//!
//! Because this library manipulates the Windows registry, code must be executed
//! as administrator or any other user with sufficient privileges.
//!
//! Errors will have an [`ErrorKind`] of either:
//! - `PermissionDenied` for insufficient privileges, or
//! - `NotFound` for operations on missing keys and values.
//!
//! [`ErrorKind`]: https://doc.rust-lang.org/std/io/enum.ErrorKind.html

pub use entry::*;
pub use utils::toggle_classic_menu;

mod entry;
mod path;
mod utils;
