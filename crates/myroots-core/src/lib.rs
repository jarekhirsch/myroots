// MyRoots © 2026 Jarek Hirsch, https://hirsch.lt
// SPDX-License-Identifier: GPL-3.0-or-later (with additional terms, see NOTICE.md)

//! MyRoots core — the genealogy engine shared by the desktop, mobile and web
//! applications and by the exported family website.

pub mod archive;
pub mod book;
pub mod charsets;
pub mod date;
pub mod db;
pub mod engine;
mod fuzz;
pub mod error;
pub mod gedcom;
pub mod health;
pub mod kinship;
pub mod life;
pub mod lifeline;
pub mod map;
pub mod metadata;
pub mod merge;
pub mod model;
pub mod ops;
pub mod privacy;
pub mod query;
pub mod site;
pub mod stats;
pub mod tree;
pub mod wp_import;

pub use db::{ChangeSet, Database, Kind, Record};
pub use engine::Engine;
pub use error::{Error, Result};
pub use life::Ctx;
