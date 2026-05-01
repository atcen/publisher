pub fn init() {
    publisher_core::init();
    println!("Publisher Typography Initialized");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typography_init() {
        init();
    }
}
