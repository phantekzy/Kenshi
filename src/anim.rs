use std::time::{Duration, Instant};

#[derive(Clone, Copy)]
pub struct Animation {
    start: Instant,
    duration: Duration,
}

impl Animation {
    pub fn started(duration_ms: u64) -> Self {
        Self {
            start: Instant::now(),
            duration: Duration::from_millis(duration_ms.max(1)),
        }
    }

    pub fn linear(&self) -> f32 {
        let t = self.start.elapsed().as_secs_f32() / self.duration.as_secs_f32();
        t.clamp(0.0, 1.0)
    }

    pub fn eased(&self) -> f32 {
        ease_out_cubic(self.linear())
    }

    pub fn eased_staggered(&self, stagger_ms: u64, total_duration_ms: u64) -> f32 {
        let elapsed = self.start.elapsed().as_millis() as i64 - stagger_ms as i64;
        if elapsed <= 0 {
            return 0.0;
        }
        let t = elapsed as f32 / total_duration_ms.max(1) as f32;
        ease_out_cubic(t.clamp(0.0, 1.0))
    }

    #[allow(dead_code)]
    pub fn is_done(&self) -> bool {
        self.start.elapsed() >= self.duration
    }
}

pub fn ease_out_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

#[derive(Clone, Copy)]
pub struct Pulse {
    start: Instant,
}

impl Pulse {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    pub fn wave(&self, period_ms: u64) -> f32 {
        let t = self.start.elapsed().as_secs_f32();
        let period = (period_ms.max(1) as f32) / 1000.0;
        let phase = (t / period) * std::f32::consts::TAU;
        (phase.sin() * 0.5) + 0.5
    }
}
