use flow_finance_ai::run;

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("fatal error: {err:#}");
        std::process::exit(1);
    }
}
