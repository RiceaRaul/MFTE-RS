// Helper function to get filename with proper borrowing
pub fn get_filename_with_default(
    provided: Option<&str>,
    default_fn: impl FnOnce() -> String,
) -> String {
    match provided {
        Some(name) => name.to_string(),
        None => default_fn(),
    }
}