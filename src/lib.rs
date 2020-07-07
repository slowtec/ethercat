// Part of ethercat-rs. Copyright 2018-2020 by the authors.
// This work is dual-licensed under Apache 2.0 and MIT terms.

#[macro_use]
extern crate num_derive;

use ethercat_sys as ec;

mod master;
mod types;

pub use self::{
    master::{Domain, Master, MasterAccess, SlaveConfig},
    types::*,
};
