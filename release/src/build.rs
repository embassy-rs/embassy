use anyhow::Result;

use crate::cargo::{CargoArgsBuilder, CargoBatchBuilder};

pub(crate) fn build(ctx: &crate::Context, crate_name: Option<&str>) -> Result<()> {
    let mut batch_builder = CargoBatchBuilder::new();

    // Process either specific crate or all crates
    let crates_to_build: Vec<_> = if let Some(name) = crate_name {
        // Build only the specified crate
        if let Some(krate) = ctx.crates.get(name) {
            vec![krate]
        } else {
            return Err(anyhow::anyhow!("Crate '{}' not found", name));
        }
    } else {
        // Build all crates
        ctx.crates.values().collect()
    };

    // Process selected crates and add their build configurations to the batch
    for krate in crates_to_build {
        for config in &krate.configs {
            let mut args_builder = CargoArgsBuilder::new()
                .subcommand("build")
                .arg("--release")
                .arg(format!("--manifest-path={}/Cargo.toml", krate.path.to_string_lossy()));

            if let Some(ref target) = config.target {
                args_builder = args_builder.target(target);
            }

            if !config.features.is_empty() {
                args_builder = args_builder.features(&config.features);
            }

            batch_builder.add_command(args_builder.build());
        }
    }

    // Execute the cargo batch command
    let batch_args = batch_builder.build();
    crate::cargo::run(&batch_args, &ctx.root)?;

    Ok(())
}
