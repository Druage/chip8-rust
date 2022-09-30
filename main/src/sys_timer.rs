use std::time::Instant;

pub struct SysTimer {
    target_fps: u64,
}

impl SysTimer {
    pub fn new(target_fps: u64) -> SysTimer {
        SysTimer {
            target_fps,
        }
    }

    pub fn pause_until_target_reached(&self) {
        let mut now = Instant::now();

        loop {
            let micro_secs_elapsed: u64 = now.elapsed().subsec_nanos() as u64 / 1_000;

            if micro_secs_elapsed >= self.target_fps {
                now = Instant::now();
                break;
            }
        }
    }
}
