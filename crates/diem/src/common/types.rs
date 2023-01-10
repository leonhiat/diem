// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::common::utils::to_common_result;
use async_trait::async_trait;
use std::fmt::Debug;
use thiserror::Error;

use serde::Serialize;

/// Trait for all CLI commands
#[async_trait]
pub trait Command<T: Serialize + Send>: Sized + Send {
    /// Returns the name of the command
    fn command_name(&self) -> &'static str;

    /// Returns a result from the executed command
    async fn execute(self) -> Result<T, CliError>;

    /// Returns a result from the executed command serialized as JSON
    async fn execute_serialized(self) -> Result<String, String> {
        to_common_result(self.execute().await).await
    }
}

// CLI Errors for logging
#[derive(Debug, Error)]
pub enum CliError {
    #[error("Aborted command")]
    AbortedError,
    #[error("Invalid arguments: {0}")]
    CommandArgumentError(String),
    #[error("Unable to load config: {0} {1}")]
    ConfigLoadError(String, String),
    #[error("Unable to find config {0}, have you run `diem init`?")]
    ConfigNotFoundError(String),
}

impl CliError {
    pub fn to_str(&self) -> &'static str {
        match self {
            CliError::AbortedError => "AbortedError",
            CliError::CommandArgumentError(_) => "CommandArgumentError",
            CliError::ConfigLoadError(_, _) => "ConfigLoadError",
            CliError::ConfigNotFoundError(_) => "ConfigNotFoundError",
        }
    }
}
