// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, fmt};

fn main() {

    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with(
            fmt::layer()
                .with_timer(fmt::time::ChronoLocal::rfc_3339()) 
                .with_target(true)      
                .with_level(true)       
                .with_ansi(true)        
                .compact(),             
        )
        .init();

    tracing::info!("MC Starter Client started");
    mc_starter_lib::run()
}
