// This file has intentional clippy violations
pub fn test_absurd_comparison() -> bool {
    let x = 5;
    // Absurd comparison - always true
    x > -1
}

pub fn test_unnecessary_cast() {
    let x: i32 = 10;
    let y = x as i32; // Unnecessary cast
}

pub fn test_unused_import() {
    use std::collections::HashMap;
    // HashMap is imported but never used
}

pub fn test_single_match() {
    match Some(42) {
        Some(value) => println!("{}", value),
        None => (),
    }
}
