mod settings;
mod formater;
pub mod cli;

use pyo3::prelude::*;
use std::env;

#[pyfunction]
fn run_cli(py: Python, args: Option<Vec<String>>) -> PyResult<()> {
    let args_vec: Vec<String> = match args {
        Some(v) => v,
        None => {
            env::args_os()
                .skip(1)
                .map(|os| os.into_string().unwrap_or_else(|os| os.to_string_lossy().into_owned()))
                .collect()
        }
    };

    py.allow_threads(|| {
        cli::main(args_vec);
    });

    Ok(())
}

#[pymodule]
fn sqlformater(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(run_cli, m)?)?;
    Ok(())
}