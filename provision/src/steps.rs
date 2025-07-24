use std::error;
use std::fs;
use std::io::prelude::*;
use std::path;
use std::process;
use tokio::process as t_process;
use tokio::time;

use async_trait::async_trait;
use sys_mount;
use tokio;

use crate::config;

const MOUNT_DIR: &str = "mount";
const IMG_MOUNT_DIR: &str = "img";
const INSTANCE_MOUNT_DIR: &str = "instance";
const ROOTFS_MOUNT_DIR: &str = "rootfs";
const BOOT_MOUNT_DIR: &str = "boot";

/// Represents a single step in provisioning an instance.
#[async_trait]
pub trait Step {
    /// Returns the name of the step.
    fn name(&self) -> String;

    /// Runs the step's provisioning logic.
    async fn run(
        &self,
        workspace_spec: &config::WorkspaceConfig,
        instance_spec: &config::InstanceConfig,
    ) -> Result<(), Box<dyn error::Error>>;

    /// Cleans up the step's provisioning logic.
    async fn cleanup(
        &self,
        workspace_spec: &config::WorkspaceConfig,
        instance_spec: &config::InstanceConfig,
    ) -> ();
}

fn output_or_err(output: process::Output) -> Result<String, Box<dyn error::Error>> {
    if !output.status.success() {
        let stdout = String::from_utf8(output.stdout)?;
        let stderr = String::from_utf8(output.stderr)?;

        println!(
            "error running command.
stderr: '{}'
stdout: '{}'",
            stderr, stdout,
        );

        Err(output.status.to_string().into())
    } else {
        let res = String::from_utf8(output.stdout)?;

        Ok(res)
    }
}

fn write_to_path(path: &[&str], contents: String) -> Result<(), Box<dyn error::Error>> {
    let pathbuf: path::PathBuf = path.iter().collect();

    let path_str = pathbuf.to_str().ok_or("invalid path")?;

    let mut file = fs::File::create(path_str)?;

    file.write_all(contents.as_bytes())?;

    Ok(())
}

/// Creates requisite directories for instance provisioning.
pub struct MkdirStep {}

#[async_trait]
impl Step for MkdirStep {
    fn name(&self) -> String {
        String::from("mkdir")
    }

    async fn run(
        &self,
        workspace_spec: &config::WorkspaceConfig,
        instance_spec: &config::InstanceConfig,
    ) -> Result<(), Box<dyn error::Error>> {
        let img_rootfs_mount_pb: path::PathBuf = [
            &workspace_spec.path,
            &instance_spec.id,
            MOUNT_DIR,
            IMG_MOUNT_DIR,
            ROOTFS_MOUNT_DIR,
        ]
        .iter()
        .collect();

        let img_boot_mount_pb: path::PathBuf = [
            &workspace_spec.path,
            &instance_spec.id,
            MOUNT_DIR,
            IMG_MOUNT_DIR,
            BOOT_MOUNT_DIR,
        ]
        .iter()
        .collect();

        let instance_rootfs_mount_pb: path::PathBuf = [
            &workspace_spec.path,
            &instance_spec.id,
            MOUNT_DIR,
            INSTANCE_MOUNT_DIR,
            ROOTFS_MOUNT_DIR,
        ]
        .iter()
        .collect();

        let instance_boot_mount_pb: path::PathBuf = [
            &workspace_spec.path,
            &instance_spec.id,
            MOUNT_DIR,
            INSTANCE_MOUNT_DIR,
            BOOT_MOUNT_DIR,
        ]
        .iter()
        .collect();

        let all_pbs = [
            img_rootfs_mount_pb,
            img_boot_mount_pb,
            instance_rootfs_mount_pb,
            instance_boot_mount_pb,
        ];

        for pb in all_pbs {
            fs::create_dir_all(pb.as_path())?;
        }

        Ok(())
    }

    async fn cleanup(
        &self,
        workspace_spec: &config::WorkspaceConfig,
        instance_spec: &config::InstanceConfig,
    ) -> () {
        let img_rootfs_mount_pb: path::PathBuf = [
            &workspace_spec.path,
            &instance_spec.id,
            MOUNT_DIR,
            IMG_MOUNT_DIR,
            ROOTFS_MOUNT_DIR,
        ]
        .iter()
        .collect();

        let img_boot_mount_pb: path::PathBuf = [
            &workspace_spec.path,
            &instance_spec.id,
            MOUNT_DIR,
            IMG_MOUNT_DIR,
            BOOT_MOUNT_DIR,
        ]
        .iter()
        .collect();

        let instance_rootfs_mount_pb: path::PathBuf = [
            &workspace_spec.path,
            &instance_spec.id,
            MOUNT_DIR,
            INSTANCE_MOUNT_DIR,
            ROOTFS_MOUNT_DIR,
        ]
        .iter()
        .collect();

        let instance_boot_mount_pb: path::PathBuf = [
            &workspace_spec.path,
            &instance_spec.id,
            MOUNT_DIR,
            INSTANCE_MOUNT_DIR,
            BOOT_MOUNT_DIR,
        ]
        .iter()
        .collect();

        let img_mount_pb: path::PathBuf = [
            &workspace_spec.path,
            &instance_spec.id,
            MOUNT_DIR,
            IMG_MOUNT_DIR,
        ]
        .iter()
        .collect();

        let instance_mount_pb: path::PathBuf = [
            &workspace_spec.path,
            &instance_spec.id,
            MOUNT_DIR,
            INSTANCE_MOUNT_DIR,
        ]
        .iter()
        .collect();

        let mount_pb: path::PathBuf = [&workspace_spec.path, &instance_spec.id, MOUNT_DIR]
            .iter()
            .collect();

        let instance_pb: path::PathBuf = [&workspace_spec.path, &instance_spec.id].iter().collect();

        let all_pbs = [
            img_rootfs_mount_pb,
            img_boot_mount_pb,
            instance_rootfs_mount_pb,
            instance_boot_mount_pb,
            img_mount_pb,
            instance_mount_pb,
            mount_pb,
            instance_pb,
        ];

        for pb in all_pbs {
            let path = pb.as_path();
            match fs::remove_dir(path) {
                Ok(_) => {}
                Err(e) => {
                    let path_str = path.to_str().unwrap_or("<invalid path>");
                    println!("error removing {}: {}", path_str, e)
                }
            }
        }
    }
}

/// Logs into the workspace iSCSI portal and instance iSCSI target.
pub struct LoginIscsiStep {}

#[async_trait]
impl Step for LoginIscsiStep {
    fn name(&self) -> String {
        String::from("login iSCSI")
    }

    async fn run(
        &self,
        workspace_spec: &config::WorkspaceConfig,
        instance_spec: &config::InstanceConfig,
    ) -> Result<(), Box<dyn error::Error>> {
        println!(
            "logging into {} to access target {}",
            workspace_spec.iscsi_target_ip, instance_spec.iscsi_target_iqn
        );

        let discover_output = t_process::Command::new("iscsiadm")
            .args([
                "--mode",
                "discovery",
                "--portal",
                &workspace_spec.iscsi_target_ip,
                "--type",
                "sendtargets",
            ])
            .output()
            .await?;

        output_or_err(discover_output)?;

        let login_output = t_process::Command::new("iscsiadm")
            .args([
                "--mode",
                "node",
                "--targetname",
                &instance_spec.iscsi_target_iqn,
                "--portal",
                &workspace_spec.iscsi_target_ip,
                "--login",
            ])
            .output()
            .await?;

        output_or_err(login_output)?;

        println!("sleeping for 5 seconds because iscsiadm is racy");

        time::sleep(time::Duration::from_millis(5_000)).await;

        Ok(())
    }

    async fn cleanup(
        &self,
        workspace_spec: &config::WorkspaceConfig,
        instance_spec: &config::InstanceConfig,
    ) -> () {
        println!(
            "logging out of {} and target {}",
            workspace_spec.iscsi_target_ip, instance_spec.iscsi_target_iqn
        );

        let output_result = t_process::Command::new("iscsiadm")
            .args([
                "--mode",
                "node",
                "--targetname",
                &instance_spec.iscsi_target_iqn,
                "--portal",
                &workspace_spec.iscsi_target_ip,
                "--logout",
            ])
            .output()
            .await;

        let output = match output_result {
            Ok(o) => o,
            Err(e) => {
                println!("error logging out of target: {}", e);
                return ();
            }
        };

        match output_or_err(output) {
            Ok(_) => {}
            Err(e) => {
                println!("error logging out of target: {}", e);
            }
        };
    }
}

/// Formats the iSCSI target device and creates an ext4 filesystem on the device.
pub struct PrepareRootfsStep {}

#[async_trait]
impl Step for PrepareRootfsStep {
    fn name(&self) -> String {
        String::from("prepare rootfs")
    }

    async fn run(
        &self,
        workspace_spec: &config::WorkspaceConfig,
        instance_spec: &config::InstanceConfig,
    ) -> Result<(), Box<dyn error::Error>> {
        let iscsi_dev_path = format!(
            "/dev/disk/by-path/ip-{}:3260-iscsi-{}-lun-1",
            workspace_spec.iscsi_target_ip, instance_spec.iscsi_target_iqn,
        );

        let iscsi_part_path = format!("{}-part1", iscsi_dev_path);

        println!("making GPT partition table on {}", iscsi_dev_path);

        let mklabel_output = t_process::Command::new("parted")
            .args(["--script", &iscsi_dev_path, "mklabel", "gpt"])
            .output()
            .await?;

        output_or_err(mklabel_output)?;

        println!("making partition on {}", iscsi_dev_path);

        let mkpart_output = t_process::Command::new("parted")
            .args([
                "--script",
                "--align",
                "optimal",
                &iscsi_dev_path,
                "mkpart",
                "primary",
                "ext4",
                "0%",
                "100%",
            ])
            .output()
            .await?;

        output_or_err(mkpart_output)?;

        let mount_path_pb: path::PathBuf = [
            &workspace_spec.path,
            &instance_spec.id,
            MOUNT_DIR,
            INSTANCE_MOUNT_DIR,
            ROOTFS_MOUNT_DIR,
        ]
        .iter()
        .collect();

        let mount_path = mount_path_pb
            .to_str()
            .ok_or(String::from("invalid mount path"))?;

        println!("sleeping for 5 seconds because iscsiadm is racy");

        println!("formatting disk at {}", iscsi_part_path);

        time::sleep(time::Duration::from_millis(5_000)).await;

        let mkfs_output = t_process::Command::new("mkfs")
            .args(["-t", "ext4", &iscsi_part_path])
            .output()
            .await?;

        output_or_err(mkfs_output)?;

        println!("finding device for {}", iscsi_part_path);

        let lsblk_output = t_process::Command::new("lsblk")
            .args(["-n", "-o", "NAME", &iscsi_part_path])
            .output()
            .await?;

        let part_out = output_or_err(lsblk_output)?;

        let part_name = part_out.trim_end();

        let dev_part_path = format!("/dev/{}", part_name);

        println!("mounting {} at {}", &dev_part_path, mount_path);

        // The mount here should persist indefinitely instead of being auto-unmounted
        // on drop
        sys_mount::Mount::builder().mount(&dev_part_path, mount_path)?;

        Ok(())
    }

    async fn cleanup(
        &self,
        workspace_spec: &config::WorkspaceConfig,
        instance_spec: &config::InstanceConfig,
    ) -> () {
        let mount_path_pb: path::PathBuf = [
            &workspace_spec.path,
            &instance_spec.id,
            MOUNT_DIR,
            INSTANCE_MOUNT_DIR,
            ROOTFS_MOUNT_DIR,
        ]
        .iter()
        .collect();

        let mount_path_result = mount_path_pb
            .to_str()
            .ok_or(String::from("invalid mount path"));

        let mount_path = match mount_path_result {
            Ok(p) => p,
            Err(e) => {
                println!("error constructing mount path: {}", e);
                return ();
            }
        };

        match sys_mount::unmount(mount_path, sys_mount::UnmountFlags::DETACH) {
            Ok(_) => {}
            Err(e) => {
                println!("error unmounting {}: {}", mount_path, e);
            }
        };
    }
}

/// Mounts the Raspberry Pi `/boot/firmware` directory at the workspace NFS server.
pub struct MountBootStep {}

impl MountBootStep {
    fn remove_dir_contents(&self, path: &path::Path) -> Result<(), Box<dyn error::Error>> {
        for entry_result in fs::read_dir(path)? {
            let entry = entry_result?;
            let entry_type = entry.file_type()?;
            let entry_path = entry.path();

            if entry_type.is_dir() {
                fs::remove_dir_all(entry_path)?;
            } else {
                fs::remove_file(entry_path)?;
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Step for MountBootStep {
    fn name(&self) -> String {
        String::from("mount boot")
    }

    async fn run(
        &self,
        workspace_spec: &config::WorkspaceConfig,
        instance_spec: &config::InstanceConfig,
    ) -> Result<(), Box<dyn error::Error>> {
        let nfs_path_pb: path::PathBuf = [&workspace_spec.nfs_tftp_dir, &instance_spec.mac_addr]
            .iter()
            .collect();

        let nfs_path = nfs_path_pb.to_str().ok_or("invalid path")?;

        // The mount syscall for NFS is a little funky. This bit of code inspired by a StackOverflow post
        // seems to work.

        let nfs_mount_src = format!(":{}", nfs_path);

        let nfs_mount_addr_option = format!("addr={}", workspace_spec.nfs_server_ip);

        let mount_path_pb: path::PathBuf = [
            &workspace_spec.path,
            &instance_spec.id,
            MOUNT_DIR,
            INSTANCE_MOUNT_DIR,
            BOOT_MOUNT_DIR,
        ]
        .iter()
        .collect();

        let mount_path = mount_path_pb.to_str().ok_or("invalid path")?;

        println!("mounting {} at {}", nfs_mount_src, mount_path);

        // The mount here should persist indefinitely instead of being auto-unmounted
        // on drop
        sys_mount::Mount::builder()
            .fstype("nfs")
            .data(&nfs_mount_addr_option)
            .mount(&nfs_mount_src, mount_path)?;

        println!("removing the contents of {}", mount_path);

        // Remove the contents of the mount directory before proceeding
        self.remove_dir_contents(&mount_path_pb)?;

        Ok(())
    }

    async fn cleanup(
        &self,
        workspace_spec: &config::WorkspaceConfig,
        instance_spec: &config::InstanceConfig,
    ) -> () {
        let mount_path_pb: path::PathBuf = [
            &workspace_spec.path,
            &instance_spec.id,
            MOUNT_DIR,
            INSTANCE_MOUNT_DIR,
            BOOT_MOUNT_DIR,
        ]
        .iter()
        .collect();

        let mount_path_result = mount_path_pb
            .to_str()
            .ok_or(String::from("invalid mount path"));

        let mount_path = match mount_path_result {
            Ok(p) => p,
            Err(e) => {
                println!("error constructing mount path: {}", e);
                return ();
            }
        };

        match sys_mount::unmount(mount_path, sys_mount::UnmountFlags::DETACH) {
            Ok(_) => {}
            Err(e) => {
                println!("error unmounting {}: {}", mount_path, e);
            }
        };
    }
}

/// Updates the kernel command line to boot via iSCSI.
pub struct UpdateCmdlineStep {}

#[async_trait]
impl Step for UpdateCmdlineStep {
    fn name(&self) -> String {
        String::from("update command line")
    }

    async fn run(
        &self,
        workspace_spec: &config::WorkspaceConfig,
        instance_spec: &config::InstanceConfig,
    ) -> Result<(), Box<dyn error::Error>> {
        let rootfs_pb: path::PathBuf = [
            &workspace_spec.path,
            &instance_spec.id,
            MOUNT_DIR,
            INSTANCE_MOUNT_DIR,
            ROOTFS_MOUNT_DIR,
        ]
        .iter()
        .collect();

        let rootfs_path = rootfs_pb.to_str().ok_or("invalid root path")?;

        let fstab_pb: path::PathBuf = [&rootfs_path, "etc/fstab"].iter().collect();

        let fstab_path = fstab_pb.to_str().ok_or("invalid fstab path")?;

        let cmdline_pb: path::PathBuf = [
            &workspace_spec.path,
            &instance_spec.id,
            MOUNT_DIR,
            INSTANCE_MOUNT_DIR,
            BOOT_MOUNT_DIR,
            "cmdline.txt",
        ]
        .iter()
        .collect();

        let cmdline_path = cmdline_pb.to_str().ok_or("invalid cmdline.txt path")?;

        let findmnt_output = t_process::Command::new("findmnt")
            .args(["-n", "-o", "SOURCE", rootfs_path])
            .output()
            .await?;

        let findmnt_stdout = output_or_err(findmnt_output)?;
        let mount_source = findmnt_stdout.trim_end();

        println!("getting PARTUUID for {}", mount_source);

        let lsblk_output = t_process::Command::new("lsblk")
            .args(["-n", "-o", "PARTUUID", mount_source])
            .output()
            .await?;

        let lsblk_stdout = output_or_err(lsblk_output)?;
        let partuuid = lsblk_stdout.trim_end();

        println!("PARTUUID for {} is: {}", mount_source, partuuid);

        let fstab_sed_expr = format!(
            "s@.*/ +.*@PARTUUID={} / ext4 _netdev,noatime 0 1@;s@.*/boot/firmware +.*@{}:{}/{} /boot/firmware nfs defaults,vers=4.1,proto=tcp 0 0@",
            partuuid,
            workspace_spec.nfs_server_ip,
            workspace_spec.nfs_tftp_dir,
            instance_spec.mac_addr,
        );

        println!("updating {} with {}", fstab_path, fstab_sed_expr);

        output_or_err(
            t_process::Command::new("sed")
                .args(["-i", "-r", "-e", &fstab_sed_expr, &fstab_path])
                .output()
                .await?,
        )?;

        let cmdline_sed_expr = format!(
            "s/root=PARTUUID=[0-9a-f-]+/root=PARTUUID={}/;s/$/ ip=dhcp ISCSI_INITIATOR={} ISCSI_TARGET_NAME={} ISCSI_TARGET_IP={} rw/g",
            partuuid,
            instance_spec.iscsi_initiator_iqn,
            instance_spec.iscsi_target_iqn,
            workspace_spec.iscsi_target_ip,
        );

        println!("updating {} with {}", cmdline_path, cmdline_sed_expr);

        output_or_err(
            t_process::Command::new("sed")
                .args(["-i", "-r", "-e", &cmdline_sed_expr, &cmdline_path])
                .output()
                .await?,
        )?;

        Ok(())
    }

    async fn cleanup(
        &self,
        _workspace_spec: &config::WorkspaceConfig,
        _instance_spec: &config::InstanceConfig,
    ) -> () {
        ()
    }
}

/// Copies Raspberry Pi OS image data to the boot and rootfs mounts.
pub struct CopyDataStep {}

impl CopyDataStep {
    async fn copy_from_img(
        &self,
        img_path: &String,
        offset: u64,
        mnt_path: &path::Path,
        target_path: &path::Path,
    ) -> Result<(), Box<dyn error::Error>> {
        let mnt_path_str = mnt_path.to_str().ok_or("invalid mount path")?;

        let target_path_str = target_path.to_str().ok_or("invalid target path")?;

        println!("mounting {} at {} on {}", img_path, offset, mnt_path_str);

        // We don't actually use this value but we hold onto it
        // so we unmount on drop
        let _mount_result = sys_mount::Mount::builder()
            .explicit_loopback()
            .loopback_offset(offset)
            .flags(sys_mount::MountFlags::RDONLY)
            .mount_autodrop(img_path, mnt_path, sys_mount::UnmountFlags::DETACH)?;

        println!(
            "copying contents of {} to {}",
            mnt_path_str, target_path_str
        );

        let cp_output = t_process::Command::new("cp")
            .args(["-r", &mnt_path_str, &target_path_str])
            .output()
            .await?;

        let _ = output_or_err(cp_output)?;

        Ok(())
    }

    async fn copy_boot(
        &self,
        workspace_spec: &config::WorkspaceConfig,
        instance_spec: &config::InstanceConfig,
    ) -> Result<(), Box<dyn error::Error>> {
        let img_boot_mount_pb: path::PathBuf = [
            &workspace_spec.path,
            &instance_spec.id,
            MOUNT_DIR,
            IMG_MOUNT_DIR,
            BOOT_MOUNT_DIR,
        ]
        .iter()
        .collect();

        let mount_pb: path::PathBuf = [
            &workspace_spec.path,
            &instance_spec.id,
            MOUNT_DIR,
            INSTANCE_MOUNT_DIR,
        ]
        .iter()
        .collect();

        self.copy_from_img(
            &workspace_spec.img_path,
            workspace_spec.img_boot_offset,
            &img_boot_mount_pb,
            &mount_pb,
        )
        .await
    }

    async fn copy_rootfs(
        &self,
        workspace_spec: &config::WorkspaceConfig,
        instance_spec: &config::InstanceConfig,
    ) -> Result<(), Box<dyn error::Error>> {
        let img_rootfs_mount_pb: path::PathBuf = [
            &workspace_spec.path,
            &instance_spec.id,
            MOUNT_DIR,
            IMG_MOUNT_DIR,
            ROOTFS_MOUNT_DIR,
        ]
        .iter()
        .collect();

        let rootfs_mount_pb: path::PathBuf = [
            &workspace_spec.path,
            &instance_spec.id,
            MOUNT_DIR,
            INSTANCE_MOUNT_DIR,
        ]
        .iter()
        .collect();

        self.copy_from_img(
            &workspace_spec.img_path,
            workspace_spec.img_rootfs_offset,
            &img_rootfs_mount_pb,
            &rootfs_mount_pb,
        )
        .await
    }
}

#[async_trait]
impl Step for CopyDataStep {
    fn name(&self) -> String {
        String::from("copy data")
    }

    async fn run(
        &self,
        workspace_spec: &config::WorkspaceConfig,
        instance_spec: &config::InstanceConfig,
    ) -> Result<(), Box<dyn error::Error>> {
        self.copy_boot(workspace_spec, instance_spec).await?;
        self.copy_rootfs(workspace_spec, instance_spec).await?;

        Ok(())
    }

    async fn cleanup(
        &self,
        _workspace_spec: &config::WorkspaceConfig,
        _instance_spec: &config::InstanceConfig,
    ) -> () {
        ()
    }
}

/// Configures a username and password for the instance.
pub struct ConfigureUserAuthStep {}

#[async_trait]
impl Step for ConfigureUserAuthStep {
    fn name(&self) -> String {
        String::from("configure auth")
    }

    async fn run(
        &self,
        workspace_spec: &config::WorkspaceConfig,
        instance_spec: &config::InstanceConfig,
    ) -> Result<(), Box<dyn error::Error>> {
        let userconf_path = [
            &workspace_spec.path,
            &instance_spec.id,
            MOUNT_DIR,
            INSTANCE_MOUNT_DIR,
            BOOT_MOUNT_DIR,
            "userconf.txt",
        ];

        let userconf_contents = format!("{}\n", instance_spec.user_password);

        write_to_path(&userconf_path, userconf_contents)?;

        let authorized_keys_path = [
            &workspace_spec.path,
            &instance_spec.id,
            MOUNT_DIR,
            INSTANCE_MOUNT_DIR,
            ROOTFS_MOUNT_DIR,
            "root/.ssh/authorized_keys",
        ];

        let authorized_keys_contents = format!("{}\n", instance_spec.root_ssh_key);

        write_to_path(&authorized_keys_path, authorized_keys_contents)
    }

    async fn cleanup(
        &self,
        _workspace_spec: &config::WorkspaceConfig,
        _instance_spec: &config::InstanceConfig,
    ) -> () {
        ()
    }
}

/// Configures the hostname for the instance.
pub struct ConfigureHostnameStep {}

impl ConfigureHostnameStep {
    fn configure_etc_hostname(
        &self,
        workspace_spec: &config::WorkspaceConfig,
        instance_spec: &config::InstanceConfig,
    ) -> Result<(), Box<dyn error::Error>> {
        let hostname_path = [
            &workspace_spec.path,
            &instance_spec.id,
            MOUNT_DIR,
            INSTANCE_MOUNT_DIR,
            ROOTFS_MOUNT_DIR,
            "etc/hostname",
        ];

        let hostname_contents = format!("{}\n", instance_spec.id);

        write_to_path(&hostname_path, hostname_contents)?;

        Ok(())
    }

    async fn configure_etc_hosts(
        &self,
        workspace_spec: &config::WorkspaceConfig,
        instance_spec: &config::InstanceConfig,
    ) -> Result<(), Box<dyn error::Error>> {
        let hosts_pb: path::PathBuf = [
            &workspace_spec.path,
            &instance_spec.id,
            MOUNT_DIR,
            INSTANCE_MOUNT_DIR,
            ROOTFS_MOUNT_DIR,
            "etc/hosts",
        ]
        .iter()
        .collect();

        let hosts_path = hosts_pb.to_str().ok_or("invalid /etc/hosts path")?;

        let hosts_sed_expr = format!("s/(.*)raspberrypi(.*?)$/\\1{}\\2/g", instance_spec.id);

        let sed_output = t_process::Command::new("sed")
            .args(["-i", "-r", "-e", &hosts_sed_expr, &hosts_path])
            .output()
            .await?;

        // We don't actually care about the output here, but we do care if the command failed
        output_or_err(sed_output)?;

        Ok(())
    }
}

#[async_trait]
impl Step for ConfigureHostnameStep {
    fn name(&self) -> String {
        String::from("configure hostname")
    }

    async fn run(
        &self,
        workspace_spec: &config::WorkspaceConfig,
        instance_spec: &config::InstanceConfig,
    ) -> Result<(), Box<dyn error::Error>> {
        self.configure_etc_hostname(workspace_spec, instance_spec)?;

        self.configure_etc_hosts(workspace_spec, instance_spec)
            .await
    }

    async fn cleanup(
        &self,
        _workspace_spec: &config::WorkspaceConfig,
        _instance_spec: &config::InstanceConfig,
    ) -> () {
        ()
    }
}

/// Does nothing on its own; this signals the completion of provisioning.
pub struct FinishStep {}

#[async_trait]
impl Step for FinishStep {
    fn name(&self) -> String {
        String::from("finish")
    }

    async fn run(
        &self,
        _workspace_spec: &config::WorkspaceConfig,
        _instance_spec: &config::InstanceConfig,
    ) -> Result<(), Box<dyn error::Error>> {
        println!("done!");

        Ok(())
    }

    async fn cleanup(
        &self,
        _workspace_spec: &config::WorkspaceConfig,
        _instance_spec: &config::InstanceConfig,
    ) -> () {
        ()
    }
}

/// Echoes a message.
pub struct EchoStep {
    pub msg: &'static str,
}

#[async_trait]
impl Step for EchoStep {
    fn name(&self) -> String {
        format!("echo {}", self.msg)
    }

    async fn run(
        &self,
        _workspace_spec: &config::WorkspaceConfig,
        _instance_spec: &config::InstanceConfig,
    ) -> Result<(), Box<dyn error::Error>> {
        println!("{}: running {}", self.name(), self.msg);

        match self.msg {
            "err" => Err("error".into()),
            _ => Ok(()),
        }
    }

    async fn cleanup(
        &self,
        _workspace_spec: &config::WorkspaceConfig,
        _instance_spec: &config::InstanceConfig,
    ) -> () {
        println!("{}: cleaning up {}", self.name(), self.msg);
    }
}
