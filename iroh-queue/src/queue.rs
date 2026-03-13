use std::collections::{BTreeMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};

use iroh::EndpointId;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::error::{QueueError, Result};

/// The lifecycle state of a job.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobState {
    Pending,
    Claimed {
        consumer: EndpointId,
        claimed_at: Instant,
    },
    Completed,
    Failed {
        reason: String,
    },
}

/// A job in the queue.
#[derive(Debug, Clone)]
pub struct Job {
    pub id: Uuid,
    pub payload: Vec<u8>,
    pub priority: u8,
    pub created_at: Instant,
    pub state: JobState,
    pub producer: EndpointId,
}

/// Thread-safe, in-memory job store with FIFO ordering.
#[derive(Debug, Clone)]
pub struct JobStore {
    inner: Arc<RwLock<JobStoreInner>>,
}

#[derive(Debug)]
struct JobStoreInner {
    jobs: BTreeMap<Uuid, Job>,
    pending_queue: VecDeque<Uuid>,
}

impl JobStore {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(JobStoreInner {
                jobs: BTreeMap::new(),
                pending_queue: VecDeque::new(),
            })),
        }
    }

    /// Add a new job to the store in `Pending` state. Returns the created job.
    pub async fn enqueue(&self, payload: Vec<u8>, priority: u8, producer: EndpointId) -> Job {
        let job = Job {
            id: Uuid::new_v4(),
            payload,
            priority,
            created_at: Instant::now(),
            state: JobState::Pending,
            producer,
        };
        let mut inner = self.inner.write().await;
        let id = job.id;
        inner.jobs.insert(id, job.clone());
        inner.pending_queue.push_back(id);
        job
    }

    /// Attempt to claim a pending job. Returns the job if successful.
    pub async fn try_claim(&self, job_id: Uuid, consumer: EndpointId) -> Result<Job> {
        let mut inner = self.inner.write().await;
        let job = inner
            .jobs
            .get(&job_id)
            .ok_or(QueueError::JobNotFound(job_id))?;

        if job.state != JobState::Pending {
            return Err(QueueError::AlreadyClaimed(job_id));
        }

        let job = inner.jobs.get_mut(&job_id).unwrap();
        job.state = JobState::Claimed {
            consumer,
            claimed_at: Instant::now(),
        };
        let claimed_job = job.clone();
        inner.pending_queue.retain(|id| *id != job_id);
        Ok(claimed_job)
    }

    /// Mark a claimed job as completed.
    pub async fn ack(&self, job_id: Uuid) -> Result<()> {
        let mut inner = self.inner.write().await;
        let job = inner
            .jobs
            .get_mut(&job_id)
            .ok_or(QueueError::JobNotFound(job_id))?;

        match &job.state {
            JobState::Claimed { .. } => {
                job.state = JobState::Completed;
                Ok(())
            }
            _ => Err(QueueError::Connection(format!(
                "job {job_id} is not in Claimed state"
            ))),
        }
    }

    /// Mark a claimed job as failed and re-enqueue it as Pending.
    pub async fn nack(&self, job_id: Uuid, reason: String) -> Result<()> {
        let mut inner = self.inner.write().await;
        let job = inner
            .jobs
            .get_mut(&job_id)
            .ok_or(QueueError::JobNotFound(job_id))?;

        match &job.state {
            JobState::Claimed { .. } => {
                job.state = JobState::Pending;
                inner.pending_queue.push_back(job_id);
                tracing::info!(%job_id, %reason, "job nacked and re-enqueued");
                Ok(())
            }
            _ => Err(QueueError::Connection(format!(
                "job {job_id} is not in Claimed state"
            ))),
        }
    }

    /// Look up a job by ID.
    pub async fn get(&self, job_id: Uuid) -> Option<Job> {
        let inner = self.inner.read().await;
        inner.jobs.get(&job_id).cloned()
    }

    /// Find claimed jobs that have exceeded the timeout and re-enqueue them.
    /// Returns the IDs of reaped jobs (for re-broadcasting).
    pub async fn reap_stale(&self, timeout: Duration) -> Vec<Uuid> {
        let mut inner = self.inner.write().await;
        let now = Instant::now();
        let mut reaped = Vec::new();

        for job in inner.jobs.values_mut() {
            if let JobState::Claimed { claimed_at, .. } = &job.state {
                if now.duration_since(*claimed_at) > timeout {
                    tracing::warn!(job_id = %job.id, "reaping stale claimed job");
                    job.state = JobState::Pending;
                    reaped.push(job.id);
                }
            }
        }

        for id in &reaped {
            inner.pending_queue.push_back(*id);
        }

        reaped
    }

    /// Return the number of pending jobs.
    pub async fn pending_count(&self) -> usize {
        let inner = self.inner.read().await;
        inner.pending_queue.len()
    }
}
