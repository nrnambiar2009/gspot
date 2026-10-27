//! Stand-in for `apply.rs` on platforms without a replace backend yet (gate L2).

use crate::ax::AXElement;
use anyhow::{anyhow, Result};
use serde::Serialize;

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Method {
    Ax,
    Paste,
    /// Edited the panel's own text box.
    Panel,
}

#[derive(Clone, Debug, Serialize)]
pub struct ApplyResult {
    pub method: Method,
    pub new_text: String,
}

pub fn replace_range(_el: &AXElement, _pid: i32, _loc: usize, _len: usize, _replacement: &str, _before: &str) -> Result<ApplyResult> {
    Err(anyhow!("Replacing text in other apps is not available on this platform yet."))
}
