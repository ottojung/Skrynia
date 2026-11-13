mod bench;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    bench::run().await
}
