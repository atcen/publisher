#[tokio::main]
async fn main() {
    publisher_core::init();
    println!("Publisher Backend Running");
}
