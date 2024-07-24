
use crate::{models::WorkflowData, workflow_runner::WorkflowRunner};

pub struct WorkflowService {}

impl WorkflowService {
    pub fn execute_manually(workflow_data: WorkflowData, all_nodes: Vec<(String, String)>) {
        WorkflowRunner::run(&workflow_data, all_nodes);
    }
}
