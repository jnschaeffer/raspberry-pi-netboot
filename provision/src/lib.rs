use std::sync::{Arc, OnceLock};

use rayon::prelude::*;

pub struct InstanceSpec {
    pub id: String,
    pub iscsi_target_ip: String,
    pub iscsi_target_iqn: String,
    pub mac_addr: String,
}

pub trait Step {
    fn run(&self, spec: &InstanceSpec) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

pub struct StepGraph {
    nodes: Vec<Box<dyn Step + Sync>>,
    edges: Vec<Vec<usize>>,
}

impl StepGraph {
    pub fn new() -> StepGraph {
        let nodes = Vec::new();
        let edges = Vec::new();

        StepGraph{nodes, edges}
    }

    pub fn add_node(&mut self, step: impl Step + Sync + 'static) -> usize {
        let len = self.nodes.len();
        let b: Box<dyn Step + Sync> = Box::new(step);

        self.nodes.push(b);
        self.edges.push(Vec::new());

        len
    }

    pub fn add_edge(&mut self, from: usize, to: usize) -> Result<(), &'static str> {
        let edges = self.edges.get_mut(from).ok_or("invalid index")?;
        self.nodes.get(to).ok_or("invalid index")?;

        edges.push(to);

        Ok(())
    }

    pub fn run(&self, spec: &InstanceSpec, step_idx: usize) -> Result<(), Arc<dyn std::error::Error + Send + Sync>> {
        let mut results: Vec<StepResult> = Vec::with_capacity(self.nodes.len());

        for _ in &self.nodes {
            results.push(StepResult::new());
        }

        self.run_step(spec, results.as_slice(), step_idx)
    }

    fn run_step(&self, spec: &InstanceSpec, results: &[StepResult], step_idx: usize) -> Result<(), Arc<dyn std::error::Error + Send + Sync>> {
        let step = self.nodes.get(step_idx).expect("what???");

        let deps = self.edges.get(step_idx).expect("no edges found");

        deps.into_par_iter()
            .try_for_each(|dep| self.run_step(spec, results, dep.clone()))?;

        let result = results.get(step_idx).expect("no result found for step");

        let r = result.lock.get_or_init(|| {
            match step.run(spec) {
                Ok(()) => Ok(()),
                Err(e) => Err(e.into()),
            }
        }
        );

        r.clone()
    }
}

struct StepResult {
    lock: OnceLock<Result<(), Arc<dyn std::error::Error + Send + Sync>>>,
}

impl StepResult {
    fn new() -> StepResult {
        StepResult{
            lock: OnceLock::new(),
        }
    }
}

