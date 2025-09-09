// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use std::fmt;
use std::process::ExitCode;

pub type CargoStylusResult = Result<(), CargoStylusError>;

#[derive(Debug)]
pub struct CargoStylusError {
    error: eyre::Error,
    exit_code: ExitCode,
}

impl CargoStylusError {
    pub fn exit_code(&self) -> ExitCode {
        self.exit_code
    }
}

impl fmt::Display for CargoStylusError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.error.fmt(f)
    }
}

impl From<std::io::Error> for CargoStylusError {
    fn from(err: std::io::Error) -> Self {
        Self {
            error: err.into(),
            exit_code: ExitCode::FAILURE,
        }
    }
}

impl From<eyre::Error> for CargoStylusError {
    fn from(error: eyre::Error) -> Self {
        Self {
            error,
            exit_code: ExitCode::FAILURE,
        }
    }
}

impl From<stylus_tools::Error> for CargoStylusError {
    fn from(err: stylus_tools::Error) -> Self {
        Self {
            error: err.into(),
            exit_code: ExitCode::FAILURE,
        }
    }
}

impl From<stylus_tools::core::build::BuildError> for CargoStylusError {
    fn from(err: stylus_tools::core::build::BuildError) -> Self {
        Self {
            error: err.into(),
            exit_code: ExitCode::FAILURE,
        }
    }
}

impl From<stylus_tools::core::check::CheckError> for CargoStylusError {
    fn from(err: stylus_tools::core::check::CheckError) -> Self {
        Self {
            error: err.into(),
            exit_code: ExitCode::FAILURE,
        }
    }
}

impl From<stylus_tools::core::deployment::DeploymentError> for CargoStylusError {
    fn from(err: stylus_tools::core::deployment::DeploymentError) -> Self {
        Self {
            error: err.into(),
            exit_code: ExitCode::FAILURE,
        }
    }
}

impl From<stylus_tools::core::network::NetworkError> for CargoStylusError {
    fn from(err: stylus_tools::core::network::NetworkError) -> Self {
        Self {
            error: err.into(),
            exit_code: ExitCode::FAILURE,
        }
    }
}
