use std::time::{Duration, Instant};

pub(crate) struct FPSCounter {
    frame_count: u32,
    last_time: Instant,
}

impl FPSCounter {
    fn new() -> Self {
        FPSCounter {
            frame_count: 0,
            last_time: Instant::now(),
        }
    }

    fn update(&mut self) -> Option<f32> {
        self.frame_count += 1;
        let elapsed = self.last_time.elapsed();
        if elapsed >= Duration::from_secs(1) {
            let fps = self.frame_count as f32 / elapsed.as_secs_f32();
            self.frame_count = 0;
            self.last_time = Instant::now();
            Some(fps)
        } else {
            None
        }
    }
}
