//! The main entry point.

use command::{Cli, Parser, TuiError};

#[tokio::main]
async fn main() -> Result<(), TuiError> {
    // Needed to use `tokio-console` in debug mode
    // #[cfg(debug_assertions)]
    // console_subscriber::init();
    let parsed = Cli::<String>::parse();
    parsed.execute().await
}
