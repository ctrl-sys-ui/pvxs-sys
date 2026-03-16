// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
// Example demonstrating PVXS logging configuration
//
// This shows how to filter PVXS internal log messages, particularly
// benign TCP disconnect errors (socket error 10054) that occur when
// clients close connections after one-shot operations like `pvput`.

use pvxs_sys::{NTScalarMetadataBuilder, Server};
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("PVXS Logging Configuration Example\n");

    // ============================================================================
    // Method 1: Suppress specific logger programmatically
    // ============================================================================
    println!("Method 1: Programmatic filtering");
    println!("  Suppressing pvxs.tcp.io errors (socket disconnects)");
    pvxs_sys::set_logger_level("pvxs.tcp.io", "CRIT")?;

    // You can configure multiple loggers:
    // pvxs_sys::set_logger_level("pvxs.server", "WARN")?;
    // pvxs_sys::set_logger_level("pvxs.*", "INFO")?; // Wildcards work

    // ============================================================================
    // Method 2: Environment variable (alternative to Method 1)
    // ============================================================================
    // Set before running:
    //   Windows: $env:PVXS_LOG="pvxs.tcp.io=CRIT"
    //   Linux:   export PVXS_LOG="pvxs.tcp.io=CRIT"
    //
    // Then call:
    // pvxs_sys::configure_logging_from_env()?;
    //
    // Multiple loggers:
    //   PVXS_LOG="pvxs.tcp.io=CRIT,pvxs.server=WARN"
    //   PVXS_LOG="pvxs.*=INFO"  // Set all internal loggers

    // ============================================================================
    // Log Levels (from highest to lowest priority)
    // ============================================================================
    // CRIT  - Only critical errors (effectively silences most messages)
    // ERR   - Errors (default for all loggers)
    // WARN  - Warnings
    // INFO  - Informational messages
    // DEBUG - Detailed debug output

    println!("\nStarting server with filtered logging...\n");

    let server = Server::start_isolated()?;
    println!("Server started on TCP port: {}", server.tcp_port());

    // Create a simple test PV
    server.create_pv_double("test:pv", 42.0, NTScalarMetadataBuilder::new())?;

    println!("\nTest with external client:");
    println!("  pvput -s test:pv 100.0");
    println!("\nSocket error 10054 messages should now be suppressed!");
    println!("Press Ctrl+C to exit\n");

    // Keep server running
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
