use std::error;
use std::fs;
use std::path;

use serde;
use serde::de;
use serde_json;

/// A configuration for a given instance provisioning run.
pub struct Config {
    /// The path to the workspace configuration JSON file.
    pub workspace_config_path: path::PathBuf,
    /// The path to the directory of instance configuration JSON files.
    pub instances_config_dir: path::PathBuf,
}

impl Config {
    /// Builds a configuration from the given command line arguments.
    pub fn build(args: &[String]) -> Result<Config, Box<dyn error::Error>> {
        let workspace_config_path = args.get(1).ok_or("missing workspace config path")?.into();

        let instances_config_dir = args.get(2).ok_or("missing instances config dir")?.into();

        Ok(Config {
            workspace_config_path,
            instances_config_dir,
        })
    }
}

/// A configuration for a workspace. A workspace is a group of instances with some shared configuration.
#[derive(serde::Deserialize)]
pub struct WorkspaceConfig {
    /// The root path to use when mounting instance and image devices.
    pub path: String,
    /// The path to the Raspberry Pi OS image to use.
    pub img_path: String,
    /// The offset for the rootfs partition in the image in bytes. Use `fdisk -l` to find this.
    pub img_rootfs_offset: u64,
    /// The offset for the boot partition in the image in bytes. Use `fdisk -l` to find this.
    pub img_boot_offset: u64,
    /// The iSCSI target IP address. Used for mounting root partitions.
    pub iscsi_target_ip: String,
    /// The NFS server IP address. Used for mounting TFTP boot partitions.
    pub nfs_server_ip: String,
    /// The TFTP directory on the NFS server.
    pub nfs_tftp_dir: String,
}

/// A configuration for an instance. An instance is a single Raspberry Pi machine.
#[derive(serde::Deserialize)]
pub struct InstanceConfig {
    /// The ID of the instance. Used to determine the hostname.
    pub id: String,
    /// The iSCSI initiator IQN. Used when mounting the root filesystem.
    pub iscsi_initiator_iqn: String,
    /// The iSCSI target IQN. Used to determine which target to mount as the root filesystem.
    pub iscsi_target_iqn: String,
    /// The MAC address for the Raspberry Pi in the form `aa-bb-cc-dd-ee-ff`.
    pub mac_addr: String,
    /// The username and password to use in the form `<username>:<hash>`. Use `openssl passwd -6` to generate the hash.
    pub user_password: String,
    /// The SSH key to use for root login.
    pub root_ssh_key: String,
}

fn load_from_path<T: de::DeserializeOwned>(path: &path::Path) -> Result<T, Box<dyn error::Error>> {
    let f = fs::File::open(path)?;

    match serde_json::from_reader(f) {
        Ok(c) => Ok(c),
        Err(e) => Err(e.into()),
    }
}

/// Loads the workspace config from the given path.
pub fn load_workspace_config(path: &path::Path) -> Result<WorkspaceConfig, Box<dyn error::Error>> {
    load_from_path(path)
}

/// Loads all instance configs from the given directory path. Any file with the extension `json` is considered to be an instance config.
pub fn load_instance_configs(
    dir: &path::Path,
) -> Result<Vec<InstanceConfig>, Box<dyn error::Error>> {
    let mut paths = Vec::new();

    for entry_result in fs::read_dir(dir)? {
        let entry_path = entry_result?.path();
        if entry_path.is_dir() {
            continue;
        }

        if let Some(ext) = entry_path.extension() {
            if ext == "json" {
                paths.push(entry_path);
            }
        }
    }

    paths.iter().map(|f| load_from_path(f)).collect()
}
