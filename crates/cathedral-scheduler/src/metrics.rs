use prometheus::{
    register_gauge, register_counter, register_histogram, Gauge, Counter, Histogram, HistogramOpts,
};
use crate::types::SchedulingDecision;

pub struct SchedulerMetrics {
    pub workers_total: Gauge,
    pub workers_by_tier: Gauge,
    pub avg_reputation: Gauge,
    pub tasks_scheduled: Counter,
    pub tasks_failed: Counter,
    pub duration_seconds: Histogram,
    pub estimated_cost: Gauge,
}

impl SchedulerMetrics {
    pub fn new() -> Self {
        let workers_total = register_gauge!("scheduler_workers_total", "Total workers registered").unwrap_or_else(|_| Gauge::new("scheduler_workers_total", "Total workers registered").unwrap());
        let workers_by_tier = register_gauge!("scheduler_workers_by_tier", "Workers by tier").unwrap_or_else(|_| Gauge::new("scheduler_workers_by_tier", "Workers by tier").unwrap());
        let avg_reputation = register_gauge!("scheduler_avg_reputation", "Average reputation").unwrap_or_else(|_| Gauge::new("scheduler_avg_reputation", "Average reputation").unwrap());
        let tasks_scheduled = register_counter!("scheduler_tasks_scheduled_total", "Total tasks scheduled").unwrap_or_else(|_| Counter::new("scheduler_tasks_scheduled_total", "Total tasks scheduled").unwrap());
        let tasks_failed = register_counter!("scheduler_tasks_failed_total", "Total tasks failed").unwrap_or_else(|_| Counter::new("scheduler_tasks_failed_total", "Total tasks failed").unwrap());
        let duration_seconds = register_histogram!(
            HistogramOpts::new("scheduler_duration_seconds", "Scheduler decision duration")
                .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5])
        ).unwrap_or_else(|_| Histogram::with_opts(HistogramOpts::new("scheduler_duration_seconds", "Scheduler decision duration").buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5])).unwrap());
        let estimated_cost = register_gauge!("scheduler_estimated_cost_usd", "Estimated cost per task").unwrap_or_else(|_| Gauge::new("scheduler_estimated_cost_usd", "Estimated cost per task").unwrap());

        Self {
            workers_total,
            workers_by_tier,
            avg_reputation,
            tasks_scheduled,
            tasks_failed,
            duration_seconds,
            estimated_cost,
        }
    }

    pub fn record_schedule_attempt(&self) {
        self.tasks_scheduled.inc();
    }

    pub fn record_schedule_failure(&self) {
        self.tasks_failed.inc();
    }

    pub fn record_schedule_decision(&self, decision: &SchedulingDecision) {
        self.estimated_cost.set(decision.estimated_cost);
    }

    pub fn record_task_success(&self) {}
    pub fn record_task_failure(&self) {
        self.tasks_failed.inc();
    }

    pub fn update_registry_stats(&self, total: usize, gpu: usize, cpu: usize, datacenter: usize, avg_rep: f32) {
        self.workers_total.set(total as f64);
        self.workers_by_tier.set(gpu as f64);
        self.avg_reputation.set(avg_rep as f64);
    }
}
