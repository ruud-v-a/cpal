
#[cfg(all(not(windows)))]
use this_platform_is_not_supported;

#[cfg(windows)]
#[path="wasapi/mod.rs"]
mod cpal_impl;