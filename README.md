# Geist Supervisor

The **Geist Supervisor** is the central orchestrator for the Geist ecosystem. A Rust-based tool that manages the update process, debugging, and runtime control of the Geist and associated applications.

## How to install
- `cargo install geist_supervisor`: Installs the Geist Supervisor to the system.

## Key Features

1. **Unified Updates**:
   - Ensures all components (Geist binaries, Roc Camera App) are updated simultaneously to a single unified version.

2. **Update Process**:
   - Automatically verifies and applies updates for:
     - Geist binaries.
     - Roc Camera App binaries.
   - Ensures integrity with checksum and signature validation.
   - Restarts all services in the correct order after updating.

3. **Bootloader like functionality**:
   - Acts as the bootloader for the Geist application, the Roc Camera App, and any future firmware components.

## CLI Commands

The command line interface is built using [clap](https://github.com/clap-rs/clap). It should just be `geist <command>`.

### Update Commands
- `geist update <version>`: Updates all components (Geist, Roc Camera App, microcontroller firmware) to the specified version.
- `geist verify <version>`: Verifies that all artifacts for a given version are available and valid.
- `geist rollback <version>`: Rolls back to a previous known-good version.

