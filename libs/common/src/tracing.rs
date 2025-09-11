use tracing_subscriber::{ FmtSubscriber, EnvFilter };

pub fn init_tracing() {
    // Build the subscriber
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env()) // reads RUST_LOG
        .finish();

    // Make it the default subscriber
    tracing::subscriber::set_global_default(subscriber).expect("setting tracing default failed");
}
