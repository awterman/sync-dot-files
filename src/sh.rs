use core::fmt;
use std::{
    error::Error,
    io,
    process::{Command, Output},
};

#[derive(Debug)]
pub enum ShError {
    Io(io::Error),
    Output(Output),
}

impl fmt::Display for ShError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ShError::Io(e) => write!(f, "{e}"),
            ShError::Output(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                write!(f, "{stderr}")
            }
        }
    }
}

impl Error for ShError {}

pub fn run_cmd(s: &str) -> Result<(String, String), ShError> {
    let output = Command::new("sh").arg("-c").arg(s).output();

    match output {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                Ok((stdout, stderr))
            } else {
                Err(ShError::Output(output))
            }
        }
        Err(e) => Err(ShError::Io(e)),
    }
}

pub fn run_cmd_stdout_only(s: &str) -> Result<String, ShError> {
    let output = Command::new("sh").arg("-c").arg(s).output();

    match output {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                Ok(stdout)
            } else {
                Err(ShError::Output(output))
            }
        }
        Err(e) => Err(ShError::Io(e)),
    }
}

macro_rules! cmd {
    ($($arg:tt)*) => {{
        crate::sh::run_cmd_stdout_only(&format!($($arg)*))
    }};
}

pub(crate) use cmd;
