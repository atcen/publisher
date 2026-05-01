pub fn init() {
    publisher_core::init();
    println!("Publisher Export PDF Initialized");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_pdf_init() {
        init();
    }
}
