//! Shared UI helpers (colours, button style, small components).

use gpui::{Hsla, Rgba, rgb, rgba};

// ---------------------------------------------------------------------------
// Colour palette
// ---------------------------------------------------------------------------

pub const BG_APP: u32 = 0x0f1117;        // main window background
pub const BG_CARD: u32 = 0x161b27;       // card / section background
pub const BG_INPUT: u32 = 0x0f1117;      // input fields
pub const BG_BTN_PRIMARY: u32 = 0x2b6cb0;
pub const BG_BTN_SECONDARY: u32 = 0x2d3748;
pub const BG_BADGE: u32 = 0x2d3748;
pub const BG_BADGE_SPLIT: u32 = 0x2b4b7e;
pub const BG_PROGRESS: u32 = 0x2d3748;
pub const BG_PROGRESS_FILL: u32 = 0x3182ce;
pub const BG_PROGRESS_DONE: u32 = 0x38a169;
pub const BG_ERROR: u32 = 0x2d1b1b;
pub const BG_SUCCESS: u32 = 0x1a2d1e;
pub const BG_BADGE_DONE: u32 = 0x1a2d1e;
pub const BG_BADGE_RUNNING: u32 = 0x2d2b00;
pub const BG_PILL: u32 = 0x2b4b7e;

pub const TEXT_TITLE: u32 = 0x90cdf4;    // blue headings
pub const TEXT_BODY: u32 = 0xe2e8f0;     // primary body
pub const TEXT_MUTED: u32 = 0xa0aec0;
pub const TEXT_DIM: u32 = 0x718096;
pub const TEXT_FAINT: u32 = 0x4a5568;
pub const TEXT_GREEN: u32 = 0x68d391;
pub const TEXT_YELLOW: u32 = 0xf6e05e;
pub const TEXT_RED: u32 = 0xfc8181;
pub const TEXT_BLUE: u32 = 0x63b3ed;
pub const TEXT_PILL: u32 = 0x90cdf4;
pub const TEXT_BADGE_DONE: u32 = 0x68d391;
pub const TEXT_BADGE_RUNNING: u32 = 0xf6e05e;
pub const TEXT_CODE: u32 = 0x68d391;
pub const BORDER: u32 = 0x2d3748;
pub const BORDER_DONE: u32 = 0x276749;
pub const BORDER_ERROR: u32 = 0x742a2a;
pub const BORDER_FOCUS: u32 = 0x4a90d9;
pub const TEXT_ERROR: u32 = 0xfc8181;
pub const TEXT_SUCCESS: u32 = 0x68d391;
