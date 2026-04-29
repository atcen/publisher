pub fn init() {
    publisher_core::init();
    println!("Publisher Export EPUB Initialized");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_epub_init() {
        init();
    }
}
