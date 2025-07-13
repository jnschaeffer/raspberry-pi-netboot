use std::{thread, time};
use std::path::PathBuf;
use std::process::{Command, Output};
use provision::{InstanceSpec, Step, StepGraph};

fn output_or_err(output: Output) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    if !output.status.success() {
        let b: Box<dyn std::error::Error + Send + Sync> = output.status.to_string().into();

        Err(b)
    } else {
        let res = String::from_utf8(output.stdout)?;

        Ok(res)
    }
}

struct MountDeviceStep {
    dev_path: &'static str,
    workspace_path: &'static str,
    mount_path: &'static str,
}

impl Step for MountDeviceStep {
    fn run(&self, spec: &InstanceSpec) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let path_buf: PathBuf = [self.workspace_path, &spec.id, self.mount_path].iter().collect();

        let mount_path = path_buf.as_path().to_str().ok_or("invalid path")?;

        println!("mounting {} at {}", self.dev_path, mount_path);

        Ok(())
    }
}

struct LoginIscsiStep {
}

impl Step for LoginIscsiStep {
    fn run(&self, spec: &InstanceSpec) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("logging into {} to access target {}", spec.iscsi_target_ip, spec.iscsi_target_iqn);

        Ok(())
    }
}

struct PrepareRootfsStep {
    workspace_path: &'static str,
    mount_path: &'static str,
}

impl Step for PrepareRootfsStep {
    fn run(&self, spec: &InstanceSpec) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let iscsi_dev_path = format!(
            "/dev/disk/by-path/ip-{}:3260-iscsi-{}-lun-1",
            spec.iscsi_target_ip,
            spec.iscsi_target_iqn,
        );

        let mount_path_pb: PathBuf = [self.workspace_path, self.mount_path].iter().collect();
        let mount_path = mount_path_pb.as_path().to_str().ok_or(String::from("invalid mount path"))?;

        println!("formatting disk at {}", iscsi_dev_path);

        thread::sleep(time::Duration::from_millis(1000));

        println!("mounting disk at {}", mount_path);

        Ok(())
    }
}

struct UpdatePartuuidStep {
    workspace_path: &'static str,
    rootfs_mount_path: &'static str,
    boot_mount_path: &'static str,
}

impl Step for UpdatePartuuidStep {
    fn run(&self, spec: &InstanceSpec) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let rootfs_pb: PathBuf = [self.workspace_path, self.rootfs_mount_path].iter().collect();
        let rootfs_path = rootfs_pb.as_path().to_str().ok_or("invalid root path")?;
        let findmnt_output = Command::new("findmnt")
            .args([
                "-n",
                "-o",
                "TARGET",
                rootfs_path,
            ])
            .output()?;
        let mount_target = output_or_err(findmnt_output)?;

        println!("getting PARTUUID for {}", mount_target);

        Ok(())
    }
}

struct CopyDataStep {
    workspace_path: &'static str,
    src_path: &'static str,
    dest_path: &'static str
}

impl Step for CopyDataStep {
    fn run(&self, _spec: &InstanceSpec) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let src_abs_pb: PathBuf = [self.workspace_path, self.src_path].iter().collect();
        let dest_abs_pb: PathBuf = [self.workspace_path, self.dest_path].iter().collect();

        let src_abs_path = src_abs_pb.as_path().to_str().ok_or("invalid source path")?;
        let dest_abs_path = dest_abs_pb.as_path().to_str().ok_or("invalid dest path")?;
        
        println!("copying contents of {} to {}", src_abs_path, dest_abs_path);

        Ok(())
    }
}

struct FinishStep {
}

impl Step for FinishStep {
    fn run(&self, _spec: &InstanceSpec) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("done!");

        Ok(())
    }
}

fn main() -> Result<(), String> {
    let mut graph = StepGraph::new();

    let mount_img_boot_step = graph.add_node(MountDeviceStep{
        workspace_path: "/home/builder/hax",
        dev_path: "/dev/loop0p1",
        mount_path: "img/boot",
    });

    let mount_img_rootfs_step = graph.add_node(MountDeviceStep{
        workspace_path: "/home/builder/hax",
        dev_path: "/dev/loop0p2",
        mount_path: "img/root",
    });

    let login_iscsi_step = graph.add_node(LoginIscsiStep{});

    let prepare_rootfs_step = graph.add_node(PrepareRootfsStep{
        workspace_path: "/home/builder/hax",
        mount_path: "mount/rootfs",
    });

    let copy_boot_step = graph.add_node(CopyDataStep{
        workspace_path: "/home/builder/hax",
        src_path: "img/boot",
        dest_path: "my/mac/addr",
    });

    let update_partuuid_step = graph.add_node(UpdatePartuuidStep{
        workspace_path: "/home/builder/hax",
        boot_mount_path: "my/mac/addr",
        rootfs_mount_path: "mount/rootfs",
    });
    
    let copy_rootfs_step = graph.add_node(CopyDataStep{
        workspace_path: "/home/builder/hax",
        src_path: "img/root",
        dest_path: "mount/rootfs",
    });

    let finish_step = graph.add_node(FinishStep{});

    graph.add_edge(finish_step, copy_rootfs_step)?;
    graph.add_edge(finish_step, copy_boot_step)?;
    graph.add_edge(finish_step, update_partuuid_step)?;
    graph.add_edge(prepare_rootfs_step, login_iscsi_step)?;
    graph.add_edge(copy_rootfs_step, prepare_rootfs_step)?;
    graph.add_edge(prepare_rootfs_step, mount_img_rootfs_step)?;
    graph.add_edge(update_partuuid_step, prepare_rootfs_step)?;
    graph.add_edge(update_partuuid_step, copy_boot_step)?;
    graph.add_edge(copy_boot_step, mount_img_boot_step)?;

    let spec = InstanceSpec{
        id: String::from("node1"),
        iscsi_target_ip: String::from("192.168.2.127"),
        iscsi_target_iqn: String::from("my::iqn"),
        mac_addr: String::from("a-b-c"),
    };

    graph.run(&spec, finish_step).or_else(|e| Err(e.to_string()))
}
