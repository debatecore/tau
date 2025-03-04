pub fn get_optional_value_to_be_patched(
    old_value: Option<String>,
    new_value: Option<String>,
) -> Option<String> {
    if new_value.is_some() {
        new_value
    } else {
        old_value
    }
}
