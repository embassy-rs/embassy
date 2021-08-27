use std::collections::HashMap;

use gen_features::{generate_cargo_toml_file, load_chip_list};

fn main() {
    let chip_list = load_chip_list();
    update_cargo_file("../embassy-stm32/Cargo.toml", &chip_list);
}

/// Update a Cargo.toml file
///
/// Update the content between "# BEGIN GENERATED FEATURES" and "# END GENERATED FEATURES"
/// with the given content
fn update_cargo_file(path: &str, new_contents: &HashMap<String, Vec<String>>) {
    let previous_text = std::fs::read_to_string(path).unwrap();
    let new_text = generate_cargo_toml_file(&previous_text, new_contents);
    std::fs::write(path, new_text).unwrap();
}
