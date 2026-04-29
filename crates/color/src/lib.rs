pub fn init() {
    publisher_core::init();
    println!("Publisher Color Initialized");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_init() {
        init();
    }
}
