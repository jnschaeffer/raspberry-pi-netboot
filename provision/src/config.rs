use std::error;
use std::fs;
use std::path;

use serde;
use serde::de;
use serde_json;

pub struct Config {
    pub workspace_config_path: path::PathBuf,
    pub instances_config_dir: path::PathBuf,
}

impl Config {
    pub fn build(args: &[String]) -> Result<Config, Box<dyn error::Error>> {
        let workspace_config_path = args.get(1).ok_or("missing workspace config path")?.into();

        let instances_config_dir = args.get(2).ok_or("missing instances config dir")?.into();

        Ok(Config {
            workspace_config_path,
            instances_config_dir,
        })
    }
}

#[derive(serde::Deserialize)]
pub struct WorkspaceConfig {
    pub path: String,
    pub img_path: String,
    pub img_rootfs_offset: u64,
    pub img_boot_offset: u64,
    pub iscsi_target_ip: String,
    pub nfs_server_ip: String,
    pub nfs_tftp_dir: String,
}

#[derive(serde::Deserialize)]
pub struct InstanceConfig {
    pub id: String,
    pub iscsi_initiator_iqn: String,
    pub iscsi_target_iqn: String,
    pub mac_addr: String,
    pub user_password: String,
    pub root_ssh_key: String,
}

fn load_from_path<T: de::DeserializeOwned>(path: &path::Path) -> Result<T, Box<dyn error::Error>> {
    let f = fs::File::open(path)?;

    match serde_json::from_reader(f) {
        Ok(c) => Ok(c),
        Err(e) => Err(e.into()),
    }
}

pub fn load_workspace_config(path: &path::Path) -> Result<WorkspaceConfig, Box<dyn error::Error>> {
    load_from_path(path)
}

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
