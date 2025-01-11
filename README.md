# Geist Supervisor

The **Geist Supervisor** is the central orchestrator for the Geist ecosystem. A Rust-based tool that manages the update process, debugging, and runtime control of the Geist and associated applications.

## Key Features

1. **Unified Updates**:
   - Ensures all components (Geist binaries, Roc Camera App, microcontroller firmware) are updated simultaneously to a single unified version.

2. **Update Process**:
   - Automatically verifies and applies updates for:
     - Geist binaries.
     - Roc Camera App binaries.
     - Microcontroller firmware.
   - Ensures integrity with checksum and signature validation.
   - Restarts all services in the correct order after updating.

3. **Bootloader and Self-Update**:
   - Acts as the bootloader for the Geist application, the Roc Camera App, and any future firmware components.
   - The only application that can self-update over the network.

## Workflow

### Unified Update Workflow
1. **Version Check**:
   - The Roc Camera App checks its version against the network repository (e.g., Artifactory).
   - If an update is available, it requests the Geist Supervisor to handle the update.

2. **Initiating Update**:
   - The Geist Supervisor downloads all artifacts (Geist binaries, Roc Camera App binaries, microcontroller firmware) for the requested version.

3. **Verification**:
   - Validates the integrity of all downloaded artifacts using checksums and cryptographic signatures.

4. **Update Application**:
   - Replaces the binaries for the Geist and Roc Camera App.
   - Flashes microcontrollers (if applicable).

5. **Service Restart**:
   - Restarts components in the correct order:
     - Flash microcontrollers first.
     - Restart the Geist software.
     - Restart the Roc Camera App.

### Debugging and Runtime Management
- Start, stop, and manage nodes in Geist.
- Echo topics to the console for debugging.

## CLI Commands

The command line interface is built using [clap](https://github.com/clap-rs/clap). It should just be `geist <command>`.

### Update Commands
- `geist update <version>`: Updates all components (Geist, Roc Camera App, microcontroller firmware) to the specified version.
- `geist verify <version>`: Verifies that all artifacts for a given version are available and valid.
- `geist rollback <version>`: Rolls back to a previous known-good version.
- `geist update-self`: Updates the Geist Supervisor itself to the latest version.

## Directory Structure for Updates

Artifacts are hosted in a central location (e.g., Artifactory, S3 bucket) with the following structure:
- TBD
