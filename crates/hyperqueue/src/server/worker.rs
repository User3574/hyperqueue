use chrono::Utc;
use tako::gateway::LostWorkerReason;
use tako::worker::WorkerConfiguration;

use crate::server::worker::WorkerState::Offline;
use crate::transfer::messages::{WorkerExitInfo, WorkerInfo};
use crate::WorkerId;

pub enum WorkerState {
    Online,
    Offline(WorkerExitInfo),
}

pub struct Worker {
    worker_id: WorkerId,
    state: WorkerState,
    pub(crate) configuration: WorkerConfiguration,
}

impl Worker {
    pub fn new(worker_id: WorkerId, configuration: WorkerConfiguration) -> Self {
        Worker {
            worker_id,
            configuration,
            state: WorkerState::Online,
        }
    }

    pub fn worker_id(&self) -> WorkerId {
        self.worker_id
    }

    pub fn configuration(&self) -> &WorkerConfiguration {
        &self.configuration
    }

    pub fn set_offline_state(&mut self, reason: LostWorkerReason) {
        self.state = Offline(WorkerExitInfo {
            ended_at: Utc::now(),
            reason,
        });
    }

    pub fn is_running(&self) -> bool {
        matches!(self.state, WorkerState::Online)
    }

    pub fn make_info(&self) -> WorkerInfo {
        WorkerInfo {
            id: self.worker_id,
            configuration: self.configuration.clone(),
            ended: match &self.state {
                WorkerState::Online => None,
                Offline(d) => Some(d.clone()),
            },
        }
    }
}
