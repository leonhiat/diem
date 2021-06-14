// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{HealthCheckError, Node, NodeExt, Validator};
use anyhow::{anyhow, Context, Result};
use diem_config::config::NodeConfig;
use diem_logger::{debug, warn};
use diem_types::{account_address::AccountAddress, PeerId};
use std::{
    env,
    fs::{self, OpenOptions},
    path::{Path, PathBuf},
    process::{Child, Command},
    str::FromStr,
};
use url::Url;

#[derive(Debug)]
struct Process(Child);

impl Drop for Process {
    // When the Process struct goes out of scope we need to kill the child process
    fn drop(&mut self) {
        // check if the process has already been terminated
        match self.0.try_wait() {
            // The child process has already terminated, perhaps due to a crash
            Ok(Some(_)) => {}

            // The process is still running so we need to attempt to kill it
            _ => {
                self.0.kill().expect("Process wasn't running");
                self.0.wait().unwrap();
            }
        }
    }
}

#[derive(Debug)]
pub struct LocalNode {
    process: Option<Process>,
    node_id: String,
    peer_id: AccountAddress,
    directory: PathBuf,
    config: NodeConfig,
    diem_node_bin_path: PathBuf,
}

impl LocalNode {
    pub fn new(diem_node_bin_path: &Path, node_id: String, directory: PathBuf) -> Result<Self> {
        let config_path = directory.join("node.yaml");
        let config = NodeConfig::load(&config_path)
            .with_context(|| format!("Failed to load NodeConfig from file: {:?}", config_path))?;
        let peer_id = config
            .peer_id()
            .ok_or_else(|| anyhow!("unable to retrieve PeerId from config"))?;

        Ok(Self {
            process: None,
            node_id,
            peer_id,
            directory,
            config,
            diem_node_bin_path: PathBuf::from(diem_node_bin_path),
        })
    }

    pub fn config_path(&self) -> PathBuf {
        self.directory.join("node.yaml")
    }

    pub fn log_path(&self) -> PathBuf {
        self.directory.join("log")
    }

    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    pub fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    pub fn start(&mut self) -> Result<()> {
        // Ensure log file exists
        let log_file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(self.log_path())?;

        // Start node process
        let mut node_command = Command::new(self.diem_node_bin_path.as_path());
        node_command
            .current_dir(&self.directory)
            .arg("-f")
            .arg(self.config_path());
        if env::var("RUST_LOG").is_err() {
            // Only set our RUST_LOG if its not present in environment
            node_command.env("RUST_LOG", "debug");
        }
        node_command.stdout(log_file.try_clone()?).stderr(log_file);
        let process = node_command.spawn().with_context(|| {
            format!(
                "Error launching node process with binary: {:?}",
                self.diem_node_bin_path.as_path()
            )
        })?;

        self.process = Some(Process(process));

        Ok(())
    }

    pub fn stop(&mut self) {
        self.process = None;
    }

    pub fn port(&self) -> u16 {
        self.config.json_rpc.address.port()
    }

    pub fn debug_port(&self) -> u16 {
        self.config
            .debug_interface
            .admission_control_node_debug_port
    }

    pub fn config(&self) -> &NodeConfig {
        &self.config
    }

    pub fn get_log_contents(&self) -> Result<String> {
        fs::read_to_string(self.log_path()).map_err(Into::into)
    }

    pub fn health_check(&mut self) -> Result<(), HealthCheckError> {
        debug!("Health check on node '{}'", self.node_id);

        if let Some(p) = &mut self.process {
            match p.0.try_wait() {
                // This would mean the child process has crashed
                Ok(Some(status)) => {
                    debug!("Node '{}' crashed with: {}", self.node_id, status);
                    return Err(HealthCheckError::NotRunning);
                }

                // This is the case where the node is still running
                Ok(None) => {}

                // Some other unknown error
                Err(e) => {
                    return Err(HealthCheckError::Unknown(e.into()));
                }
            }
        } else {
            warn!("Node '{}' is stopped", self.node_id);
            return Err(HealthCheckError::NotRunning);
        }

        self.debug_client()
            .get_node_metrics()
            .map(|_| ())
            .map_err(HealthCheckError::RpcFailure)
    }
}

impl Node for LocalNode {
    fn peer_id(&self) -> PeerId {
        self.peer_id()
    }

    fn node_id(&self) -> crate::NodeId {
        todo!()
    }

    fn json_rpc_endpoint(&self) -> reqwest::Url {
        let ip = self.config().json_rpc.address.ip();
        let port = self.config().json_rpc.address.port();
        Url::from_str(&format!("http://{}:{}/v1", ip, port)).expect("Invalid URL.")
    }

    fn debug_endpoint(&self) -> Url {
        Url::parse(&format!("http://localhost:{}", self.debug_port())).unwrap()
    }

    fn config(&self) -> &NodeConfig {
        self.config()
    }

    fn start(&mut self) -> Result<()> {
        self.start()
    }

    fn stop(&mut self) -> Result<()> {
        self.stop();
        Ok(())
    }

    fn clear_storage(&mut self) -> Result<()> {
        todo!()
    }

    fn health_check(&mut self) -> Result<(), HealthCheckError> {
        self.health_check()
    }
}

impl Validator for LocalNode {}
