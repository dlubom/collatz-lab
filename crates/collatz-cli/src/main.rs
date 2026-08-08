#![forbid(unsafe_code)]

use std::ffi::OsString;
use std::process::ExitCode;

use collatz_experiments::{Catalog, materialize_configuration, run_configuration};

const USAGE: &str = "Usage:\n  collatz-cli catalog validate <catalog.jsonl>\n  collatz-cli experiment plan <experiment.json> --output <plan.json>\n  collatz-cli experiment run <experiment.json> --output <results.jsonl>";

fn main() -> ExitCode {
    match execute(std::env::args_os().skip(1).collect()) {
        Ok(message) => {
            println!("{message}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}\n\n{USAGE}");
            ExitCode::from(2)
        }
    }
}

fn execute(arguments: Vec<OsString>) -> Result<String, String> {
    let arguments = arguments
        .into_iter()
        .map(|argument| {
            argument
                .into_string()
                .map_err(|_| "arguments must be valid UTF-8".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;

    match arguments.as_slice() {
        [area, action, path] if area == "catalog" && action == "validate" => {
            let catalog = Catalog::load_jsonl(path)
                .map_err(|error| format!("{}: {error}", error.status_code()))?;
            Ok(format!(
                "catalog valid: {} version-1 records, 0 invalid records",
                catalog.len()
            ))
        }
        [area, action, configuration, flag, output]
            if area == "experiment" && action == "plan" && flag == "--output" =>
        {
            let plan = materialize_configuration(configuration)
                .map_err(|error| format!("{}: {error}", error.status_code()))?;
            plan.write(output)
                .map_err(|error| format!("invalid_input: {error}"))?;
            Ok(format!(
                "plan materialized: configuration_id={}, inputs={}",
                plan.configuration_id,
                plan.inputs.len()
            ))
        }
        [area, action, configuration, flag, output]
            if area == "experiment" && action == "run" && flag == "--output" =>
        {
            let run = run_configuration(configuration)
                .map_err(|error| format!("{}: {error}", error.status_code()))?;
            run.write_jsonl(output)
                .map_err(|error| format!("{}: {error}", error.status_code()))?;
            Ok(format!(
                "experiment complete: configuration_id={}, run_id={}, results={}",
                run.plan.configuration_id,
                run.run_id,
                run.records.len()
            ))
        }
        _ => Err("invalid command".into()),
    }
}
