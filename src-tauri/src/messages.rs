//! Centralized message codes and helpers for backend-to-frontend surfaces.
//! Use codes to let the frontend map localized strings while keeping an English
//! fallback in the detail payload.

/// Known error/success codes (string values are stable for frontend mapping).
pub const CODE_GAME_TARGET_NOT_SET: &str = "ERR_GAME_TARGET_NOT_SET";
pub const CODE_GAME_TARGET_INVALID: &str = "ERR_GAME_TARGET_INVALID";
pub const CODE_MOD_NOT_FOUND: &str = "ERR_MOD_NOT_FOUND";
pub const CODE_MOD_ALREADY_EXISTS: &str = "ERR_MOD_ALREADY_EXISTS";
pub const CODE_SOURCE_PATH_MISSING: &str = "ERR_SOURCE_PATH_MISSING";
pub const CODE_PATH_NOT_FOUND: &str = "ERR_PATH_NOT_FOUND";
pub const CODE_METADATA_REQUIRED: &str = "NEEDS_METADATA";

/// Formats a code with an English fallback detail.
pub fn code_error(code: &'static str, detail: impl Into<String>) -> String {
    format!("[{}] {}", code, detail.into())
}

/// Formats a code without detail (for cases where code alone is meaningful).
pub fn code_only(code: &'static str) -> String {
    format!("[{}]", code)
}
