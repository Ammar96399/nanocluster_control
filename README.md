## nanocluster_control

Control power and fan settings for a Sipeed NanoCluster from your terminal via an LPI3H. This CLI connects to your cluster node at slot 1 (an LPI3H in my case) over SSH and can boot/shutdown nodes, query power status, and manage the controller’s fan mode and speed.

### Highlights

- Power control: boot, shutdown, status for individual nodes or the whole cluster
- Fan control on the controller: enable/disable mode and set speed (0–4)
- Simple configuration stored in a TOML file via `confy`

---

## Quick start

1) Install Rust (stable) and Cargo.

2) Build and run:

```sh
cargo build --release
./target/release/nanocluster_control --help
```

3) Configure your cluster (see Configuration below) and ensure you have passwordless SSH access to the controller.

---

## Install

From source:

```sh
git clone https://github.com/Ammar96399/nanocluster_control.git
cd nanocluster_control
cargo install --path .
```

This will place `nanocluster_control` in your Cargo bin directory (usually `~/.cargo/bin`). Alternatively, use the built binary in `./target/release/` after a release build.

---

## Configuration

This tool uses `confy` to store a TOML configuration. On Linux the default path is typically:

`~/.config/nanocluster_control/nanocluster_control.toml`

The binary also computes the exact path internally via `confy::get_configuration_file_path("nanocluster_control", "nanocluster_control")`.

Minimal schema:

```toml
# ~/.config/nanocluster_control/nanocluster_control.toml

ssh_username = "your_ssh_username"

[cluster]
_ip_address = "192.0.2.10" # optional/unused marker in current code

[[cluster.nodes]]
_ip_address = "192.0.2.11"    # node IP or hostname
hostname    = "controller"     # host reachable via SSH
model       = "CM5"            # one of: CM4, CM5, LPI3H
slot_number = 1                 # controller is slot 1 (reserved)

[[cluster.nodes]]
_ip_address = "192.0.2.12"
hostname    = "node-02"
model       = "CM5"
slot_number = 2

[[cluster.nodes]]
_ip_address = "192.0.2.13"
hostname    = "node-03"
model       = "CM4"
slot_number = 3

# ... add more nodes as needed
```

Important:

- Slot 1 is treated as the cluster controller by this tool. Power operations on slot 1 are intentionally blocked.
- SSH is used to reach the controller (`slot_number == 1`) and sometimes nodes, so set `ssh_username` accordingly and ensure key-based auth works.
- Fan controls write to sysfs via `sudo tee`. You’ll need passwordless sudo for the SSH user on the controller for these paths:
	- `/sys/class/thermal/thermal_zone2/mode` (fan mode)
	- `/sys/class/thermal/cooling_device0/cur_state` (fan speed)

---

## Usage

General form:

```sh
nanocluster_control <COMMAND> [--node <n|all>] [--fan-mode <enabled|disabled>] [--fan-speed <0-4>]
```

Commands:

- `BOOT`       — Boot one node (`--node <n>`) or all nodes (`--node all` or default)
- `SHUTDOWN`   — Shutdown one node or all nodes
- `STATUS`     — Print power reachability status for one node or all nodes
- `FANMODE`    — Set controller fan mode to enabled/disabled (requires `--fan-mode`)
- `FANSPEED`   — Set controller fan speed state 0–4 (requires `--fan-speed`)

Options:

- `--node <n|all>`
	- Node selector. Defaults to `all`.
- `--fan-mode <enabled|disabled>`
	- Used with `FANMODE`. Defaults to `disabled` if not specified.
- `--fan-speed <0-4>`
	- Used with `FANSPEED`. Defaults to `4` if not specified.

Notes:

- `STATUS` uses `ping` to check reachability; ensure `ping` is available on the host running this tool.
- The controller (slot 1) is skipped for power actions.

---

## Examples

Boot all nodes:

```sh
nanocluster_control boot --node all
```

Shutdown node 3:

```sh
nanocluster_control shutdown --node 3
```

Check status of all nodes:

```sh
nanocluster_control status
```

Enable fan control mode on the controller:

```sh
nanocluster_control fanmode --fan-mode enabled
```

Set fan speed level to 2:

```sh
nanocluster_control fanspeed --fan-speed 2
```

---

## Logging

This project uses `env_logger`. Set `RUST_LOG` to control verbosity:

```sh
RUST_LOG=info nanocluster_control STATUS
RUST_LOG=debug nanocluster_control BOOT --node 2
```

---

## Troubleshooting

- SSH fails or hangs:
	- Verify `ssh_username` and hostnames in the config.
	- Ensure your SSH key is accepted by the controller.
	- First connection may add host keys; re-run if needed.

- `sudo: a password is required` on fan commands:
	- Configure passwordless sudo for the SSH user on the controller for the sysfs paths used by this tool.

- STATUS always shows “off”:
	- Ensure `ping` is installed and not blocked by firewall/ICMP rules.
	- Confirm node hostnames/IPs are correct.

- Slot 1 not affected by BOOT/SHUTDOWN:
	- This is by design; slot 1 is treated as the controller and excluded from power actions.

---

## Development

Build and run locally:

```sh
cargo check
cargo test   # (no tests yet)
cargo run -- STATUS
```

The project targets Rust edition 2024 and uses:

- clap (derive) for CLI parsing
- tokio for async runtime
- openssh for SSH sessions
- serde for config serialization
- confy for config management
- env_logger/log for logging

---

## License

MIT OR Apache-2.0. See the `license` field in `Cargo.toml`.
