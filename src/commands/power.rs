use std::thread::sleep;

use anyhow::{self};
use log;
use openssh::{KnownHosts, Session};

use crate::Config;

pub async fn print_cluster_power_status(config: &Config) {
    for node in &config.cluster.nodes {
        let status = power_status(config, &node.slot_number).await;
        let state_str = if status { "ON" } else { "OFF" };
        println!(
            "Slot {} ({}) [{}]: {}",
            node.slot_number,
            node.hostname,
            format!("{:?}", node.model),
            state_str
        );
    }
}

pub async fn print_node_power_status(config: &Config, slot_number: i32) {
    if let Some(node) = config
        .cluster
        .nodes
        .iter()
        .find(|n| n.slot_number == slot_number)
    {
        let status = power_status(config, &slot_number).await;
        let state_str = if status { "ON" } else { "OFF" };
        println!(
            "Slot {} ({}) [{}]: {}",
            node.slot_number,
            node.hostname,
            format!("{:?}", node.model),
            state_str
        );
    } else {
        log::error!("Node with slot number {} not found", slot_number);
    }
}

pub async fn boot_all_nodes(config: &Config) {
    for node in &config.cluster.nodes {
        if node.slot_number == 1 {
            continue; // Skip controller node
        }
        let status = power_status(config, &node.slot_number).await;
        if status {
            log::info!("Node {} is already on, skipping.", node.slot_number);
            continue;
        }
        match node.model {
            crate::Model::CM5 => {
                if let Err(e) = boot_cm5_single_node(
                    &config
                        .cluster
                        .nodes
                        .iter()
                        .find(|n| n.slot_number == 1)
                        .unwrap()
                        .hostname,
                    &config.ssh_username,
                    &node.slot_number,
                )
                .await
                {
                    log::error!("Failed to boot CM5 node {}: {}", node.slot_number, e);
                }
            }
            crate::Model::CM4 => {
                if let Err(e) = boot_cm4_single_node(
                    &config
                        .cluster
                        .nodes
                        .iter()
                        .find(|n| n.slot_number == 1)
                        .unwrap()
                        .hostname,
                    &config.ssh_username,
                    &node.slot_number,
                )
                .await
                {
                    log::error!("Failed to boot CM4 node {}: {}", node.slot_number, e);
                }
            }
            crate::Model::LPI3H => {
                log::error!("Boot for LPI3H node {} not implemented!", node.slot_number);
            }
        }
    }
}

pub async fn boot_single_node(config: &Config, slot_number: i32) {
    if slot_number == 1 {
        log::error!("Cannot turn on the controller node !");
        return;
    }

    let node_power_status = power_status(&config, &7).await;

    if node_power_status {
        log::error!("Node is already on");
        return;
    }

    let cluster_controller_hostname = &config
        .cluster
        .nodes
        .iter()
        .find(|n| n.slot_number == 1)
        .unwrap()
        .hostname;
    let model = &config
        .cluster
        .nodes
        .iter()
        .find(|n| n.slot_number == slot_number)
        .unwrap()
        .model;
    match model {
        crate::Model::CM5 => {
            if let Err(e) = boot_cm5_single_node(
                cluster_controller_hostname,
                &config.ssh_username,
                &slot_number,
            )
            .await
            {
                log::error!("Failed to shutdown CM5 node: {}", e);
            }
        }
        crate::Model::CM4 => {
            if let Err(e) = boot_cm4_single_node(
                cluster_controller_hostname,
                &config.ssh_username,
                &slot_number,
            )
            .await
            {
                log::error!("Failed to shutdown CM4 node: {}", e);
            }
        }
        crate::Model::LPI3H => log::error!("No implemented yet !"),
    }
}

pub async fn shutdown_all_nodes(config: &Config) {
    for node in &config.cluster.nodes {
        if node.slot_number == 1 {
            continue; // Skip controller node
        }
        let status = power_status(config, &node.slot_number).await;
        if !status {
            log::info!("Node {} is already off, skipping.", node.slot_number);
            continue;
        }
        match node.model {
            crate::Model::CM5 => {
                if let Err(e) = shutdown_cm5_single_node(
                    &config
                        .cluster
                        .nodes
                        .iter()
                        .find(|n| n.slot_number == 1)
                        .unwrap()
                        .hostname,
                    &config.ssh_username,
                    &node.hostname,
                    &node.slot_number,
                )
                .await
                {
                    log::error!("Failed to shutdown CM5 node {}: {}", node.slot_number, e);
                }
            }
            crate::Model::CM4 => {
                if let Err(e) = shutdown_cm4_single_node(
                    &config
                        .cluster
                        .nodes
                        .iter()
                        .find(|n| n.slot_number == 1)
                        .unwrap()
                        .hostname,
                    &config.ssh_username,
                    &node.hostname,
                    &node.slot_number,
                )
                .await
                {
                    log::error!("Failed to shutdown CM4 node {}: {}", node.slot_number, e);
                }
            }
            crate::Model::LPI3H => {
                log::error!(
                    "Shutdown for LPI3H node {} not implemented!",
                    node.slot_number
                );
            }
        }
    }
}

pub async fn shutdown_single_node(config: &Config, slot_number: i32) {
    if slot_number == 1 {
        log::error!("Cannot turn off the controller node !");
        return;
    }

    let node_power_status = power_status(&config, &7).await;

    if !node_power_status {
        log::error!("Node is already off");
        return;
    }

    let cluster_controller_hostname = &config
        .cluster
        .nodes
        .iter()
        .find(|n| n.slot_number == 1)
        .unwrap()
        .hostname;
    let node_hostname = &config
        .cluster
        .nodes
        .iter()
        .find(|n| n.slot_number == slot_number)
        .unwrap()
        .hostname;
    let model = &config
        .cluster
        .nodes
        .iter()
        .find(|n| n.slot_number == slot_number)
        .unwrap()
        .model;
    match model {
        crate::Model::CM5 => {
            if let Err(e) = shutdown_cm5_single_node(
                cluster_controller_hostname,
                &config.ssh_username,
                node_hostname,
                &slot_number,
            )
            .await
            {
                log::error!("Failed to shutdown CM5 node: {}", e);
            }
        }
        crate::Model::CM4 => {
            if let Err(e) = shutdown_cm4_single_node(
                cluster_controller_hostname,
                &config.ssh_username,
                node_hostname,
                &slot_number,
            )
            .await
            {
                log::error!("Failed to shutdown CM4 node: {}", e);
            }
        }
        crate::Model::LPI3H => log::error!("No implemented yet !"),
    }
}

async fn shutdown_cm4_single_node(
    controller_hostname: &String,
    username: &String,
    hostname: &String,
    slot_number: &i32,
) -> anyhow::Result<()> {
    send_ssh_shutdown_command(&username, &hostname).await?;
    sleep(std::time::Duration::from_millis(2000));
    cm4_power_off_button(&username, &controller_hostname, &slot_number).await?;
    Ok(())
}

async fn shutdown_cm5_single_node(
    controller_hostname: &String,
    username: &String,
    hostname: &String,
    slot_number: &i32,
) -> anyhow::Result<()> {
    send_ssh_shutdown_command(&username, &hostname).await?;
    sleep(std::time::Duration::from_millis(2000));
    cm5_short_push_power_button(&username, &controller_hostname, &slot_number).await?;
    Ok(())
}

async fn boot_cm4_single_node(
    controller_hostname: &String,
    username: &String,
    slot_number: &i32,
) -> anyhow::Result<()> {
    cm4_power_on_button(&username, &controller_hostname, &slot_number).await?;
    Ok(())
}

async fn boot_cm5_single_node(
    controller_hostname: &String,
    username: &String,
    slot_number: &i32,
) -> anyhow::Result<()> {
    cm5_short_push_power_button(&username, &controller_hostname, &slot_number).await?;
    Ok(())
}

async fn send_ssh_shutdown_command(
    username: &String,
    hostname: &String,
) -> Result<(), anyhow::Error> {
    let session = Session::connect(format!("{username}@{hostname}"), KnownHosts::Add).await?;
    let output = session
        .command("sudo")
        .arg("shutdown")
        .arg("-h")
        .arg("now")
        .output()
        .await?;
    log::info!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}

async fn cm4_power_off_button(
    username: &String,
    controller_hostname: &String,
    slot_number: &i32,
) -> anyhow::Result<()> {
    let session =
        Session::connect(format!("{username}@{controller_hostname}"), KnownHosts::Add).await?;
    let output = session
        .command("sudo")
        .arg("gpioset")
        .arg("gpiochip2")
        .arg(format!("{slot_number}=0"))
        .output()
        .await
        .expect("Failed to execute gpioset command");
    log::info!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}

async fn cm4_power_on_button(
    username: &String,
    controller_hostname: &String,
    slot_number: &i32,
) -> anyhow::Result<()> {
    let session =
        Session::connect(format!("{username}@{controller_hostname}"), KnownHosts::Add).await?;
    let output = session
        .command("sudo")
        .arg("gpioset")
        .arg("gpiochip2")
        .arg(format!("{slot_number}=1"))
        .output()
        .await
        .expect("Failed to execute gpioset command");
    log::info!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}

async fn cm5_short_push_power_button(
    username: &String,
    controller_hostname: &String,
    slot_number: &i32,
) -> anyhow::Result<()> {
    let session =
        Session::connect(format!("{username}@{controller_hostname}"), KnownHosts::Add).await?;
    // Set GPIO low
    let output1 = session
        .command("sudo")
        .arg("gpioset")
        .arg("gpiochip2")
        .arg(format!("{slot_number}=0"))
        .output()
        .await
        .expect("Failed to execute gpioset command (set low)");
    log::info!("{}", String::from_utf8_lossy(&output1.stdout));

    // Sleep for 1 second
    sleep(std::time::Duration::from_secs(1));

    // Set GPIO high
    let output2 = session
        .command("sudo")
        .arg("gpioset")
        .arg("gpiochip2")
        .arg(format!("{slot_number}=1"))
        .output()
        .await
        .expect("Failed to execute gpioset command (set high)");
    log::info!("{}", String::from_utf8_lossy(&output2.stdout));

    Ok(())
}

/// Asynchronously executes the `ping` command with the specified hostname,
/// sending a single ICMP packet (`-c 1`) and waiting up to 1 second for a response (`-W 1`).
///
/// # Arguments
///
/// * `hostname` - The target host to ping.
///
/// # Returns
///
/// Returns a `tokio::process::Output` future containing the output of the ping command.
///
/// # Errors
///
/// Returns an error if the command fails to start or complete.
///
/// # Example
///
/// ```rust
/// let output = tokio::process::Command::new("ping")
///     .arg("-c")
///     .arg("1")
///     .arg("-W")
///     .arg("1")
///     .arg(hostname)
///     .output()
///     .await;
/// ```
/// Returns true if the node is reachable (powered on), false otherwise.
async fn power_status(config: &Config, slot_number: &i32) -> bool {
    if let Some(node) = config
        .cluster
        .nodes
        .iter()
        .find(|n| n.slot_number == *slot_number)
    {
        let hostname = &node.hostname;
        if let Ok(output) = tokio::process::Command::new("ping")
            .arg("-c")
            .arg("1")
            .arg("-W")
            .arg("1")
            .arg(hostname)
            .output()
            .await
        {
            return output.status.success();
        }
    }
    false
}
