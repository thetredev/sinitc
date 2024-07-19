use clap::{Parser, Subcommand};
use std::{os::unix::process::CommandExt, process::Command, time::Duration};

use crate::services::ServiceRegistry;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    ///
    /// Start all services
    ///
    #[command(verbatim_doc_comment)]
    Init { args: Vec<String> },

    ///
    /// Show status of a service and exit with exit codes
    ///   0 if service is started
    ///   1 if service is stopped
    ///
    #[command(verbatim_doc_comment)]
    Status { service: String },

    ///
    /// Start a service and exit with exit codes
    ///   0 if service started successfully
    ///   1 service exit code if start failed
    ///
    #[command(verbatim_doc_comment)]
    Start { service: String },

    ///
    /// Stop a service and exit with exit codes
    ///   0 if service stopped successfully
    ///   1 if stop failed
    ///
    #[command(verbatim_doc_comment)]
    Stop { service: String },

    ///
    /// Restart a service and exit with exit codes
    ///   0 if service restart successfully
    ///   1 if restart failed
    ///
    #[command(verbatim_doc_comment)]
    Restart { service: String },

    ///
    /// Show logs (stdout) of a service and exit with exit codes
    ///   0 if service is started
    ///   1 if service is stopped
    ///
    #[command(verbatim_doc_comment)]
    Stdout { service: String },

    ///
    /// Show logs (stderr) of a service and exit with exit codes
    ///   0 if service is started
    ///   1 if service is stopped
    ///
    #[command(verbatim_doc_comment)]
    Stderr { service: String },
}

impl Cli {
    pub fn evaluate(&self, services: &ServiceRegistry) {
        if let Some(command) = &self.command {
            match command {
                Commands::Init { args } => {
                    services.init();

                    // fix messed up terminal output
                    std::thread::sleep(Duration::from_millis(10));

                    // exec into binary
                    let mut args = args.iter();
                    Command::new(args.next().unwrap()).args(args).exec();
                }
                Commands::Status { service } => services.status(service),
                Commands::Start { service } => services.start(service),
                Commands::Stop { service } => services.stop(service),
                Commands::Restart { service } => services.restart(service),
                Commands::Stdout { service } => services.stdout(service),
                Commands::Stderr { service } => services.stderr(service),
            }
        }
    }
}
