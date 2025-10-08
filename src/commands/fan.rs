use openssh::{KnownHosts, Session};

use crate::{Config, FanMode, FanSpeed};

pub async fn fan_mode(config: &Config, fan_mode: &FanMode) -> anyhow::Result<()> {
    let username = &config.ssh_username;
    let cluster_controller_hostname = &config
        .cluster
        .nodes
        .iter()
        .find(|n| n.slot_number == 1)
        .unwrap()
        .hostname;
    let session = Session::connect(
        format!("{username}@{cluster_controller_hostname}"),
        KnownHosts::Add,
    )
    .await?;
    let output = session
        .command("sh")
        .arg("-c")
        .arg(format!(
            "echo {fan_mode} | sudo tee /sys/class/thermal/thermal_zone2/mode"
        ))
        .output()
        .await
        .expect("Failed to execute disable fan command");
    log::info!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}

pub async fn fan_speed(config: &Config, fan_speed: &FanSpeed) -> anyhow::Result<()> {
    let username = &config.ssh_username;
    let cluster_controller_hostname = &config
        .cluster
        .nodes
        .iter()
        .find(|n| n.slot_number == 1)
        .unwrap()
        .hostname;
    let session = Session::connect(
        format!("{username}@{cluster_controller_hostname}"),
        KnownHosts::Add,
    )
    .await?;
    let output = session
        .command("sh")
        .arg("-c")
        .arg(format!(
            "echo {fan_speed} | sudo tee /sys/class/thermal/cooling_device0/cur_state"
        ))
        .output()
        .await
        .expect("Failed to execute set fan speed command");
    log::info!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}
