use std::path::Path;
use std::{error::Error, fs::File, io::BufReader, path::PathBuf, process};

use jsonschema::JSONSchema;
use structopt::StructOpt;

type BoxErrorResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, StructOpt)]
#[structopt(name = "jsonschema")]
struct Cli {
    /// A path to a JSON instance (i.e. filename.json) to validate (may be specified multiple times).
    #[structopt(short = "i", long = "instance")]
    instances: Option<Vec<PathBuf>>,

    /// The JSON Schema to validate with (i.e. schema.json).
    #[structopt(parse(from_os_str), required_unless("version"))]
    schema: Option<PathBuf>,

    /// Show program's version number and exit.
    #[structopt(short = "v", long = "version")]
    version: bool,
}

pub fn main() -> BoxErrorResult<()> {
    let config = Cli::from_args();

    if config.version {
        println!(concat!("Version: ", env!("CARGO_PKG_VERSION")));
        return Ok(());
    }

    let mut success = true;
    if let Some(schema) = config.schema {
        if let Some(instances) = config.instances {
            success = validate_instances(&instances, schema)?;
        }
    }

    if !success {
        process::exit(1);
    }

    Ok(())
}

fn read_json(path: &Path) -> serde_json::Result<serde_json::Value> {
    let file = File::open(path).expect("Failed to open file");
    let reader = BufReader::new(file);
    serde_json::from_reader(reader)
}

fn validate_instances(instances: &[PathBuf], schema_path: PathBuf) -> BoxErrorResult<bool> {
    let mut success = true;

    let schema_json = read_json(&schema_path)?;
    match JSONSchema::compile(&schema_json) {
        Ok(schema) => {
            for instance in instances {
                let instance_json = read_json(instance)?;
                let validation = schema.validate(&instance_json);
                let filename = instance.to_string_lossy();
                match validation {
                    Ok(_) => println!("{} - VALID", filename),
                    Err(errors) => {
                        success = false;

                        println!("{} - INVALID. Errors:", filename);
                        for (i, e) in errors.enumerate() {
                            println!("{}. {}", i + 1, e);
                        }
                    }
                }
            }
        }
        Err(error) => {
            println!("Schema is invalid. Error: {}", error);
            success = false;
        }
    }
    Ok(success)
}
