pub fn init() {
    publisher_core::init();
    println!("Publisher Collab Initialized");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collab_init() {
        init();
    }
}
