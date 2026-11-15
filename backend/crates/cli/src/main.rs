use std::{env, error::Error, fs, process::ExitCode};

use orchestrata_application::solve_service::SolveService;
use orchestrata_solver::Problem;

fn run() -> Result<(), Box<dyn Error>> {
    let path = env::args().nth(1).ok_or("usage: orchestrata-cli <problem.json>")?;
    let source = fs::read_to_string(path)?;
    let problem: Problem = serde_json::from_str(&source)?;
    let solution = SolveService::execute(&problem).map_err(|error| format!("{error:?}"))?;
    println!("{}", serde_json::to_string_pretty(&solution)?);
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
