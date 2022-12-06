pub fn format_error(error: &str, object: &str) -> String {
    return error.replace("{}", object);
}