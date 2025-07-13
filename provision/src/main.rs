use std::{thread, time};

use provision::{InstanceSpec, Step, StepGraph};

struct MountDeviceStep {
    dev_path: &'static str,
    mount_path: &'static str,
}

impl Step for MountDeviceStep {
    fn run(&self, _spec: &InstanceSpec) -> Result<(), &'static str> {
        println!("mounting {} at {}", self.dev_path, self.mount_path);

        Ok(())
    }
}

struct FormatDiskStep {
}

impl Step for FormatDiskStep {
    fn run(&self, spec: &InstanceSpec) -> Result<(), &'static str> {
        println!("formatting disk at {}", spec.iscsi_dev_path);

        thread::sleep(time::Duration::from_millis(1000));

        Ok(())
    }
}

struct CopyDataStep {
    src_path: &'static str,
    dest_path: &'static str
}

impl Step for CopyDataStep {
    fn run(&self, _spec: &InstanceSpec) -> Result<(), &'static str> {
        println!("copying contents of {} to {}", self.src_path, self.dest_path);

        Ok(())
    }
}

struct FinishStep {
}

impl Step for FinishStep {
    fn run(&self, _spec: &InstanceSpec) -> Result<(), &'static str> {
        println!("done!");

        Ok(())
    }
}

fn main() -> Result<(), &'static str> {
    let mut graph = StepGraph::new();

    let mount_boot_step = graph.add_node(MountDeviceStep{
        dev_path: "/dev/loop0p1",
        mount_path: "/img/boot",
    });

    let mount_rootfs_step = graph.add_node(MountDeviceStep{
        dev_path: "/dev/loop0p2",
        mount_path: "/img/root",
    });

    let format_step = graph.add_node(FormatDiskStep{
    });

    let copy_boot_step = graph.add_node(CopyDataStep{
        src_path: "/img/boot",
        dest_path: "/my/mac/addr",
    });

    let copy_rootfs_step = graph.add_node(CopyDataStep{
        src_path: "/img/root",
        dest_path: "/my/rootfs",
    });

    let finish_step = graph.add_node(FinishStep{});

    graph.add_edge(finish_step, copy_rootfs_step)?;
    graph.add_edge(finish_step, copy_boot_step)?;
    graph.add_edge(copy_rootfs_step, format_step)?;
    graph.add_edge(format_step, mount_rootfs_step)?;
    graph.add_edge(copy_boot_step, mount_boot_step)?;

    let spec = InstanceSpec{
        mac_addr: "a-b-c",
        iscsi_dev_path: "/dev/foo",
    };

    graph.run(&spec, finish_step)
}
