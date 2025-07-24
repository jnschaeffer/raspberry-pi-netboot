pub mod config;
pub mod graph;
mod steps;

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

    // While in theory we can run all provisions concurrently, in practice this swamps the NAS and
    // causes odd behavior like iSCSI timeouts. Thus, we provision each machine serially instead.
    let mut results = Vec::with_capacity(instance_specs.len());

    for spec in instance_specs {
        let result = graph.run(workspace_spec, spec, finish_step).await;

        results.push(result);
    }

    results
}
