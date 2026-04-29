/// WASM bindings for browser build
pub fn init() {
    println!("Publisher WASM Initialized");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_init() {
        init();
    }
}
