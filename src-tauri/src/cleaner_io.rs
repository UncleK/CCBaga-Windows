// Cross-platform atomic replacement; private temporary files are removed on failure.
use std::path::Path;
pub(super) fn atomic_write(path: &Path, bytes: &[u8], mode: u32) -> Result<(), String> {
    crate::platform::write(path, bytes, mode)
}
