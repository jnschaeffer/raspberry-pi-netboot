use std::collections::{HashMap, HashSet};
use std::fmt;

use futures::future::join_all;
use thiserror::Error;
use tokio::sync::broadcast::{channel, Receiver, Sender};

use crate::config;
use crate::steps;

#[derive(Clone, Error, Debug)]
pub struct StepError {
    step_name: String,
    msg: String,
}

impl fmt::Display for StepError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error running {}: {}", self.step_name, self.msg)
    }
}

#[derive(Clone, Debug)]
struct VisitResult {
    node_idx: usize,
    result: Result<(), StepError>,
}

pub struct StepGraph {
    nodes: Vec<Box<dyn steps::Step>>,
    edges_fwd: Vec<Vec<usize>>,
    edges_rev: Vec<Vec<usize>>,
}

impl StepGraph {
    pub fn new() -> StepGraph {
        let nodes = Vec::new();
        let edges_fwd = Vec::new();
        let edges_rev = Vec::new();

        StepGraph {
            nodes,
            edges_fwd,
            edges_rev,
        }
    }

    pub fn add_node(&mut self, step: impl steps::Step + 'static) -> usize {
        let len = self.nodes.len();
        let b: Box<dyn steps::Step> = Box::new(step);

        self.nodes.push(b.into());
        self.edges_fwd.push(Vec::new());
        self.edges_rev.push(Vec::new());

        len
    }

    pub fn add_edge(&mut self, from: usize, to: usize) {
        let edges_fwd = self.edges_fwd.get_mut(from).expect("edges not found");
        let edges_rev = self.edges_rev.get_mut(to).expect("edges not found");

        self.nodes.get(to).expect("node not found");

        edges_fwd.push(to);
        edges_rev.push(from);
    }

    pub fn build_node_set(&self, node: usize, set: &mut HashSet<usize>) {
        set.insert(node);

        let edges_fwd = self.edges_fwd.get(node).expect("invalid index");

        edges_fwd.iter().for_each(|n| self.build_node_set(*n, set));
    }

    async fn visit(
        &self,
        workspace_spec: &config::WorkspaceConfig,
        instance_spec: &config::InstanceConfig,
        node_idx: usize,
        mut dependencies: Vec<Receiver<VisitResult>>,
        out: Sender<VisitResult>,
        visit_fn: impl AsyncFn(
            &Box<dyn steps::Step>,
            &config::WorkspaceConfig,
            &config::InstanceConfig,
        ) -> Result<(), Box<dyn std::error::Error>>,
    ) {
        let step = self.nodes.get(node_idx).expect("node not found");

        let step_name = step.name();

        println!("{}: starting", step_name);

        println!("{}: waiting for dependencies", step_name);

        let dependencies_results = join_all(dependencies.iter_mut().map(async |d| d.recv().await));

        for res in dependencies_results.await {
            println!("{}: received result", step_name);

            let v = res.unwrap();

            if let Err(e) = &v.result {
                println!(
                    "{}: received dependency error {}, returning early",
                    step_name, e
                );

                out.send(v).unwrap();

                return;
            }
        }

        let result = match visit_fn(step, workspace_spec, instance_spec).await {
            Ok(_) => Ok(()),
            Err(e) => Err(StepError {
                step_name,
                msg: e.to_string(),
            }),
        };

        let v = VisitResult { node_idx, result };

        out.send(v).unwrap();
    }

    async fn walk(
        &self,
        workspace_spec: &config::WorkspaceConfig,
        instance_spec: &config::InstanceConfig,
        node_set: &HashSet<usize>,
        neighbor_fn: impl Fn(usize) -> Vec<usize>,
        visit_fn: impl AsyncFn(
            &Box<dyn steps::Step>,
            &config::WorkspaceConfig,
            &config::InstanceConfig,
        ) -> Result<(), Box<dyn std::error::Error>>,
    ) -> HashMap<usize, VisitResult> {
        let mut result_senders: HashMap<usize, Sender<VisitResult>> =
            HashMap::with_capacity(node_set.len());

        let mut result_receivers: HashMap<usize, Receiver<VisitResult>> =
            HashMap::with_capacity(node_set.len());

        for node in node_set {
            let (r_tx, r_rx) = channel(1);

            result_senders.insert(*node, r_tx);
            result_receivers.insert(*node, r_rx);
        }

        let futures = node_set.iter().map(|node| {
            let deps = neighbor_fn(*node);

            let result_sender = result_senders
                .get(node)
                .expect("sender not found")
                .to_owned();

            let dependencies_recv: Vec<Receiver<VisitResult>> = deps
                .iter()
                .map(|n| result_senders.get(n).expect("sender not found").subscribe())
                .collect();

            self.visit(
                &workspace_spec,
                &instance_spec,
                *node,
                dependencies_recv,
                result_sender,
                &visit_fn,
            )
        });

        let joined = join_all(futures);

        let mut results = HashMap::with_capacity(node_set.len());

        joined.await;

        for node in node_set {
            let result = result_receivers
                .get_mut(node)
                .expect("receiver not found")
                .recv()
                .await
                .unwrap();

            results.insert(*node, result);
        }

        results
    }

    pub async fn run(
        &self,
        workspace_spec: &config::WorkspaceConfig,
        instance_spec: &config::InstanceConfig,
        until: usize,
    ) -> Result<(), StepError> {
        let mut node_set = &mut HashSet::new();

        self.build_node_set(until, &mut node_set);

        let run_neighbor_fn = |n: usize| -> Vec<usize> {
            self.edges_fwd
                .get(n)
                .expect("edges not found")
                .iter()
                .filter(|n| node_set.contains(n))
                .map(|n| *n)
                .collect()
        };

        let run_visit_fn =
            async |s: &Box<dyn steps::Step>,
                   wspec: &config::WorkspaceConfig,
                   ispec: &config::InstanceConfig| { s.run(wspec, ispec).await };

        let run_results = self
            .walk(
                &workspace_spec,
                &instance_spec,
                node_set,
                run_neighbor_fn,
                run_visit_fn,
            )
            .await;

        let run_result = run_results
            .get(&until)
            .expect("result not found")
            .result
            .clone();

        let visited_node_set = &mut HashSet::new();

        for v in run_results.values() {
            visited_node_set.insert(v.node_idx);
        }

        let cleanup_neighbor_fn = |n: usize| -> Vec<usize> {
            self.edges_rev
                .get(n)
                .expect("edges not found")
                .iter()
                .filter(|n| visited_node_set.contains(n))
                .map(|n| *n)
                .collect()
        };

        let cleanup_visit_fn =
            async |s: &Box<dyn steps::Step>,
                   wspec: &config::WorkspaceConfig,
                   ispec: &config::InstanceConfig| {
                s.cleanup(wspec, ispec).await;

                Ok(())
            };

        self.walk(
            &workspace_spec,
            &instance_spec,
            visited_node_set,
            cleanup_neighbor_fn,
            cleanup_visit_fn,
        )
        .await;

        run_result
    }
}
