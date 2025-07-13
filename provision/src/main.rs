use std::env;
use std::error;
use std::iter;

use tokio;

use provision;
use provision::config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    let args: Vec<String> = env::args().collect();

    let cfg = config::Config::build(&args)?;

    let workspace_spec = config::load_workspace_config(&cfg.workspace_config_path)?;

    let instance_specs = config::load_instance_configs(&cfg.instances_config_dir)?;

    let results = provision::run(&workspace_spec, &instance_specs).await;

    let mut failed = false;

    for (spec, result) in iter::zip(instance_specs, results) {
        match result {
            Ok(_) => println!("{}: ok", spec.id),
            Err(e) => {
                println!("{}: {}", spec.id, e);
                failed = true;
            }
        }
    }

    if !failed {
        Ok(())
    } else {
        Err("some instances failed to provision".into())
    }
}
