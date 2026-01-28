//! Validate command implementation

use anyhow::Result;
use console::style;
use nexus_config::{parse_agent_config, validate_config};
use std::path::Path;

pub fn execute(files: &[impl AsRef<Path>]) -> Result<()> {
    if files.is_empty() {
        anyhow::bail!("No files specified. Usage: nexus validate <file1> <file2> ...");
    }

    let mut all_valid = true;
    let mut valid_count = 0;
    let mut invalid_count = 0;

    for file in files {
        let path = file.as_ref();

        print!("Validating {}... ", path.display());

        match validate_file(path) {
            Ok(_) => {
                println!("{}", style("✓ valid").green().bold());
                valid_count += 1;
            }
            Err(e) => {
                println!("{}", style("✗ invalid").red().bold());
                eprintln!("  Error: {}", style(e).red());
                invalid_count += 1;
                all_valid = false;
            }
        }
    }

    println!();
    println!("Summary:");
    println!("  {} {}", style("✓").green(), style(format!("{} valid", valid_count)).green());

    if invalid_count > 0 {
        println!("  {} {}", style("✗").red(), style(format!("{} invalid", invalid_count)).red());
    }

    if all_valid {
        println!();
        println!("{}", style("All configurations are valid!").green().bold());
        Ok(())
    } else {
        println!();
        anyhow::bail!("Some configurations are invalid");
    }
}

fn validate_file(path: &Path) -> Result<()> {
    // Parse the configuration
    let config = parse_agent_config(path)?;

    // Validate the configuration
    validate_config(&config)?;

    Ok(())
}
