//! The replacement ladder: AX selected-text set, then pasteboard patch, each
//! verified by reading the field back (what Grammarly's TextCorruptionDetector does).

#![cfg(target_os = "macos")]

use crate::ax::{self, AXElement};
use crate::textmap::{slice_utf16, splice_utf16};
use anyhow::{anyhow, Result};
use serde::Serialize;
use std::os::raw::c_void;
use std::ptr::null;
use std::thread::sleep;
use std::time::Duration;

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

type CGEventRef = *mut c_void;
type CGEventSourceRef = *const c_void;

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventCreateKeyboardEvent(source: CGEventSourceRef, keycode: u16, keydown: bool) -> CGEventRef;
    fn CGEventSetFlags(event: CGEventRef, flags: u64);
    fn CGEventPost(tap: u32, event: CGEventRef);
}
#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFRelease(cf: *const c_void);
}

const KCG_HID_EVENT_TAP: u32 = 0;
const KCG_FLAG_MASK_COMMAND: u64 = 0x0010_0000;
const KEYCODE_V: u16 = 9;

fn post_cmd_v() {
    unsafe {
        let down = CGEventCreateKeyboardEvent(null(), KEYCODE_V, true);
        CGEventSetFlags(down, KCG_FLAG_MASK_COMMAND);
        CGEventPost(KCG_HID_EVENT_TAP, down);
        CFRelease(down as *const c_void);
        let up = CGEventCreateKeyboardEvent(null(), KEYCODE_V, false);
        CGEventSetFlags(up, KCG_FLAG_MASK_COMMAND);
        CGEventPost(KCG_HID_EVENT_TAP, up);
        CFRelease(up as *const c_void);
    }
}

fn verified(el: &AXElement, loc: usize, replacement: &str) -> Option<String> {
    let now = el.string("AXValue")?;
    let got = slice_utf16(&now, loc, loc + replacement.encode_utf16().count());
    if got == replacement {
        Some(now)
    } else {
        None
    }
}

/// Replace `len` UTF-16 units at `loc` with `replacement` in the host field.
pub fn replace_range(el: &AXElement, pid: i32, loc: usize, len: usize, replacement: &str, before: &str) -> Result<ApplyResult> {
    // Guard: the field must still contain what we think it does.
    let live = el.string("AXValue").ok_or_else(|| anyhow!("The text field is no longer available."))?;
    if live != before {
        // Tolerate if the target range still holds the original text.
        let want = slice_utf16(before, loc, loc + len);
        if slice_utf16(&live, loc, loc + len) != want {
            return Err(anyhow!("The text changed since this suggestion was made. Try again."));
        }
    }
    let expected = splice_utf16(&live, loc, len, replacement);

    // Rung 1: select the range, then set the selected text through AX.
    if el.set_range("AXSelectedTextRange", loc, len).is_ok() {
        let sel_ok = el.range("AXSelectedTextRange").map(|(l, n)| l == loc && n == len).unwrap_or(false);
        if sel_ok && el.set_string("AXSelectedText", replacement).is_ok() {
            sleep(Duration::from_millis(70));
            if let Some(new_text) = verified(el, loc, replacement) {
                let _ = el.set_range("AXSelectedTextRange", loc + replacement.encode_utf16().count(), 0);
                return Ok(ApplyResult { method: Method::Ax, new_text });
            }
        }
        // Rung 2: pasteboard patch (what Grammarly calls "hybrid replacements").
        if sel_ok || el.set_range("AXSelectedTextRange", loc, len).is_ok() {
            ax::activate_app(pid);
            sleep(Duration::from_millis(120));
            if paste_text(replacement).is_ok() {
                for _ in 0..6 {
                    sleep(Duration::from_millis(80));
                    if let Some(new_text) = verified(el, loc, replacement) {
                        return Ok(ApplyResult { method: Method::Paste, new_text });
                    }
                }
                // Some editors normalise text (smart quotes, trailing space). Accept if the whole value matches the expectation.
                if let Some(now) = el.string("AXValue") {
                    if now == expected {
                        return Ok(ApplyResult { method: Method::Paste, new_text: now });
                    }
                }
            }
        }
    }
    Err(anyhow!("This app did not accept the replacement. Try editing it directly."))
}

fn paste_text(text: &str) -> Result<()> {
    use arboard::Clipboard;
    let mut cb = Clipboard::new()?;
    let saved = cb.get_text().ok();
    cb.set_text(text.to_string())?;
    sleep(Duration::from_millis(60));
    post_cmd_v();
    sleep(Duration::from_millis(160));
    if let Some(prev) = saved {
        let _ = cb.set_text(prev);
    }
    Ok(())
}
