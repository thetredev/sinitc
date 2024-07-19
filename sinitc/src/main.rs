use clap::Parser;

use sinitc::{cli::Cli, services::ServiceRegistry};

fn main() {
    // handle command line arguments
    let services = ServiceRegistry::default();
    Cli::parse().evaluate(&services);
}
