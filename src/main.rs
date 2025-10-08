use std::fmt::Display;

use clap::Parser;

use crate::commands::{fan, power};

mod commands;

#[derive(Debug)]
pub struct Config {
    ssh_username: String,
    cluster: Cluster,
}

#[derive(Debug)]
struct Cluster {
    _ip_address: String,
    nodes: Vec<Node>,
}

#[derive(Debug)]
struct Node {
    _ip_address: String,
    hostname: String,
    model: Model,
    slot_number: i32,
}

#[derive(Debug)]
enum Model {
    CM5,
    CM4,
    LPI3H,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            ssh_username: "akazem".to_owned(),
            cluster: Cluster {
                _ip_address: "131.254.100.102".to_owned(),
                nodes: vec![
                    Node {
                        _ip_address: "131.254.100.100".to_owned(),
                        hostname: "lpi3h-0".to_owned(),
                        model: Model::LPI3H,
                        slot_number: 1,
                    },
                    Node {
                        _ip_address: "131.254.100.96".to_owned(),
                        hostname: "cm4-0".to_owned(),
                        model: Model::CM4,
                        slot_number: 2,
                    },
                    Node {
                        _ip_address: "131.254.100.29".to_owned(),
                        hostname: "cm4-1".to_owned(),
                        model: Model::CM4,
                        slot_number: 3,
                    },
                    Node {
                        _ip_address: "131.254.100.97".to_owned(),
                        hostname: "cm5-0".to_owned(),
                        model: Model::CM5,
                        slot_number: 5,
                    },
                    Node {
                        _ip_address: "131.254.100.98".to_owned(),
                        hostname: "cm5-1".to_owned(),
                        model: Model::CM5,
                        slot_number: 6,
                    },
                    Node {
                        _ip_address: "131.254.100.99".to_owned(),
                        hostname: "cm5-2".to_owned(),
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

    /// Node number to operate on, or 'all' for all nodes (default: all)
    #[clap(long = "node", value_parser = parse_node_selector, default_value = "all")]
    node: NodeSelector,

    #[clap(long = "fan-mode", value_enum, default_value = "disabled")]
    fan_mode: FanMode,

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
    let config: Config = Default::default();
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
