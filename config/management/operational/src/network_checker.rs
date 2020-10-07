// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_management::error::Error;
use libra_network_address::NetworkAddress;
use std::{
    io::{Read, Write},
    net::{TcpStream, ToSocketAddrs},
    time::Duration,
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct CheckEndpoint {
    #[structopt(long)]
    address: NetworkAddress,
}

// This will show up as an error in the logs as a bad key 07070707...
// It allows us to ensure that both connections work, and that we can see them in the logs
const INVALID_NOISE_HEADER: &[u8; 152] = &[7; 152];

impl CheckEndpoint {
    pub fn execute(self) -> Result<String, Error> {
        let addrs = self.address.to_socket_addrs().map_err(|err| {
            Error::IO(
                "Failed to resolve address from NetworkAddress".to_string(),
                err,
            )
        })?;

        let mut last_error = std::io::Error::new(std::io::ErrorKind::Other, "");

        // The problem here is the endpoint is not supposed to respond to garbage data.
        // So, we check that we get nothing from the endpoint, and that it resolves & connects with TCP
        for addr in addrs {
            eprintln!("Trying address: {}", addr);
            match TcpStream::connect_timeout(&addr, Duration::from_secs(5)) {
                Ok(mut stream) => {
                    // We should be able to write to the socket dummy data
                    if let Err(error) = stream.write(INVALID_NOISE_HEADER) {
                        eprintln!("Failed to write to address {}", error);
                        last_error = error;
                        continue;
                    }
                    let buf = &mut [0; 1];
                    match stream.read(buf) {
                        Ok(size) => {
                            if size == 0 {
                                // Connection is open, and doesn't return anything
                                // This is the closest we can get to working
                                return Ok(format!(
                                    "Accepted write and responded with nothing at {}",
                                    addr
                                ));
                            } else {
                                eprintln!(
                                    "Endpoint responded with data!  Shouldn't be a noise endpoint."
                                );
                                last_error = std::io::Error::new(
                                    std::io::ErrorKind::Other,
                                    "Responded with data when it shouldn't.",
                                )
                            }
                        }
                        Err(error) => {
                            eprintln!("Failed to read from address {}", error);
                            last_error = error
                        }
                    }
                }
                Err(error) => last_error = error,
            }
        }

        Err(Error::IO(
            "No addresses responded correctly".to_string(),
            last_error,
        ))
    }
}
