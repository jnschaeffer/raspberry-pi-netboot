pub mod config;
pub mod graph;
mod steps;

use futures::future;

/// Provisions instances as defined by all configs associated with the given workspace.
pub async fn run(
    workspace_spec: &config::WorkspaceConfig,
    instance_specs: &Vec<config::InstanceConfig>,
) -> Vec<Result<(), graph::StepError>> {
    let mut graph = graph::StepGraph::new();

    let mkdir_step = graph.add_node(steps::MkdirStep {});

    let mount_boot_step = graph.add_node(steps::MountBootStep {});

    let login_iscsi_step = graph.add_node(steps::LoginIscsiStep {});

    let prepare_rootfs_step = graph.add_node(steps::PrepareRootfsStep {});

    let copy_data_step = graph.add_node(steps::CopyDataStep {});

    let update_cmdline_step = graph.add_node(steps::UpdateCmdlineStep {});

    let configure_hostname_step = graph.add_node(steps::ConfigureHostnameStep {});

    let configure_user_auth_step = graph.add_node(steps::ConfigureUserAuthStep {});

    let finish_step = graph.add_node(steps::FinishStep {});

    graph.add_edge(finish_step, update_cmdline_step);
    graph.add_edge(finish_step, configure_hostname_step);
    graph.add_edge(finish_step, configure_user_auth_step);

    graph.add_edge(configure_user_auth_step, copy_data_step);

    graph.add_edge(configure_hostname_step, copy_data_step);

    graph.add_edge(update_cmdline_step, copy_data_step);

    graph.add_edge(copy_data_step, prepare_rootfs_step);
    graph.add_edge(copy_data_step, mount_boot_step);

    graph.add_edge(mount_boot_step, mkdir_step);

    graph.add_edge(prepare_rootfs_step, login_iscsi_step);
    graph.add_edge(prepare_rootfs_step, mkdir_step);

    let results = instance_specs
        .iter()
        .map(|spec| graph.run(workspace_spec, spec, finish_step));

    let joined = future::join_all(results);

    joined.await
}
