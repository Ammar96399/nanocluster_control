use std::{fmt::Display, io::Read};

use clap::Parser;
use serde_derive::{Deserialize, Serialize};

use crate::commands::{fan, power};

mod commands;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    ssh_username: String,
    cluster: Cluster,
}

#[derive(Debug, Serialize, Deserialize)]
struct Cluster {
    _ip_address: String,
    nodes: Vec<Node>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Node {
    _ip_address: String,
    hostname: String,
    model: Model,
    slot_number: i32,
}

#[derive(Debug, Serialize, Deserialize)]
enum Model {
    CM5,
    CM4,
    LPI3H,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            ssh_username: "".to_owned(),
            cluster: Cluster {
                _ip_address: "".to_owned(),
                nodes: vec![
                    Node {
                        _ip_address: "".to_owned(),
                        hostname: "".to_owned(),
                        model: Model::LPI3H,
                        slot_number: 1,
                    },
                    Node {
                        _ip_address: "".to_owned(),
                        hostname: "".to_owned(),
                        model: Model::CM4,
                        slot_number: 2,
                    },
                    Node {
                        _ip_address: "".to_owned(),
                        hostname: "".to_owned(),
                        model: Model::CM4,
                        slot_number: 3,
                    },
                    Node {
                        _ip_address: "".to_owned(),
                        hostname: "".to_owned(),
                        model: Model::CM5,
                        slot_number: 5,
                    },
                    Node {
                        _ip_address: "".to_owned(),
                        hostname: "".to_owned(),
                        model: Model::CM5,
                        slot_number: 6,
                    },
                    Node {
                        _ip_address: "".to_owned(),
                        hostname: "".to_owned(),
                        model: Model::CM5,
                        slot_number: 7,
                    },
                ],
            },
        }
    }
}

#[derive(Parser)]
struct Cli {
    #[clap(value_enum)]
    command: Command,

    /// Node number to operate on, or 'all' for all nodes
    #[clap(long = "node", value_parser = parse_node_selector, default_value = "all")]
    node: NodeSelector,

    /// Enable or disable the auto fan speed mode that comes with the LPI3H
    #[clap(long = "fan-mode", value_enum, default_value = "disabled")]
    fan_mode: FanMode,

    /// Manually set the fan speed. Requires the fan_mode to be set at disabled
    #[clap(long = "fan-speed", value_parser = parse_fan_speed, default_value = "4")]
    fan_speed: FanSpeed,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum FanMode {
    Disabled,
    Enabled,
}

impl Display for FanMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FanMode::Disabled => write!(f, "disabled"),
            FanMode::Enabled => write!(f, "enabled"),
        }
    }
}

#[derive(Debug, Clone)]
struct FanSpeed(i32);

impl Display for FanSpeed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn parse_fan_speed(s: &str) -> Result<FanSpeed, String> {
    let speed = s
        .parse::<i32>()
        .map_err(|_| format!("Invalid fan speed: {}", s))?;
    if speed < 0 || speed > 4 {
        Err(format!("Fan speed must be between 0 and 4, got {}", speed))
    } else {
        Ok(FanSpeed(speed))
    }
}

#[derive(Debug, Clone)]
enum NodeSelector {
    All,
    Number(i32),
}

fn parse_node_selector(s: &str) -> Result<NodeSelector, String> {
    if s.eq_ignore_ascii_case("all") {
        Ok(NodeSelector::All)
    } else {
        s.parse::<i32>()
            .map(NodeSelector::Number)
            .map_err(|_| format!("Invalid node selector: {}", s))
    }
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum Command {
    SHUTDOWN,
    BOOT,
    STATUS,
    FANMODE,
    FANSPEED,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let args = Cli::parse();
    let config: Config = confy::load("nanocluster_control", "nanocluster_control")?;
    let config_file =
        confy::get_configuration_file_path("nanocluster_control", "nanocluster_control")?;
    let mut config_file_content = String::new();

    std::fs::File::open(&config_file)
        .expect("Failed to open toml configuration file.")
        .read_to_string(&mut config_file_content)
        .expect("Failed to read toml configuration file.");

    let node_number = match args.node {
        NodeSelector::All => None,
        NodeSelector::Number(n) => Some(n),
    };

    match args.command {
        Command::SHUTDOWN => {
            if node_number != None {
                power::shutdown_single_node(&config, node_number.unwrap()).await
            } else {
                power::shutdown_all_nodes(&config).await
            }
        }
        Command::BOOT => {
            if node_number != None {
                power::boot_single_node(&config, node_number.unwrap()).await
            } else {
                power::boot_all_nodes(&config).await
            }
        }
        Command::STATUS => {
            if node_number != None {
                power::print_node_power_status(&config, node_number.unwrap()).await
            } else {
                power::print_cluster_power_status(&config).await
            }
        }
        Command::FANMODE => fan::fan_mode(&config, &args.fan_mode).await?,
        Command::FANSPEED => fan::fan_speed(&config, &args.fan_speed).await?,
    }

    Ok(())
}
