# embedded-rust-template
Template repository for Embedded Rust development

## Customizing This Template

### Changing the Target Architecture

This template is configured for `thumbv8m.main-none-eabihf`, by default, but you can modify it for other targets (i.e. `aarch64-unknown-none`):

1. **VSCode Settings**: Update the target in `.vscode/settings.json`:
   ```json
   "rust-analyzer.cargo.target": "your-target-architecture"
   ```


This configuration ensures that:
- Only the specified target architecture is analyzed, not the host platform
- Code is checked against the no_std environment

To temporarily analyze code for the host platform instead, you can remove the `rust-analyzer.cargo.target` setting.

2. **GitHub Workflows**: Modify the target in two workflow files:
   - `.github/workflows/nostd.yml`: Update the targets in the matrix:
     ```yaml
     matrix:
       target: [your-target-architecture]
     ```
   - `.github/workflows/check.yml`: If there are any target-specific checks, update them accordingly.

3. **Cargo Configuration**: If needed, you can add target-specific configuration in a `.cargo/config.toml` file.

### Converting from Binary to Library

To convert this project from a binary to a library:

1. **Cargo.toml**: Update your project structure:
   ```toml
   [lib]
   name = "your_library_name"
   ```

2. **Directory Structure**:
   - For a library, ensure you have a `src/lib.rs` file instead of `src/main.rs`
   - Move your code from `main.rs` to `lib.rs` and adjust as needed

3. **No-std Configuration**: If you're creating a no-std library, ensure you have:
   ```rust
   // In lib.rs
   #![cfg_attr(target_os = "none", no_std)]
   // Add other attributes as needed
   ```

### Project Dependencies

Update the dependencies in `Cargo.toml` based on your target platform:

```toml
[dependencies]
# Common dependencies for all targets

[target.'cfg(target_os = "none")'.dependencies]
# Dependencies for no-std targets
```
