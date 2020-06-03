use std::default::Default;
use std::ops::Add;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const DEFAULT_EPOCH: i64 = 1527811200000000000;

#[derive(Debug)]
pub struct FastIdWorker {
    time_bits: i64,
    machine_bits: i64,
    sequence_bits: i64,

    time_mask: i64,
    machine_mask: i64,
    sequence_mask: i64,

    machine_id: i64,
    sequence: i64,

    epoch: SystemTime,

    last_timestamp: i64,
}

impl FastIdWorker {
    pub fn new(machine_id: i64) -> Self {
        // time_bits: 40,
        // machine_bits: 16,
        // sequence_bits: 7,

        FastIdWorker::with_bits(40, 16, 7, machine_id)
    }

    pub fn with_bits(
        time_bits: i64,
        machine_bits: i64,
        sequence_bits: i64,
        machine_id: i64,
    ) -> Self {
        FastIdWorker::with_bits_and_epoch(
            time_bits,
            machine_bits,
            sequence_bits,
            machine_id,
            DEFAULT_EPOCH,
        )
    }

    pub fn with_bits_and_epoch(
        time_bits: i64,
        machine_bits: i64,
        sequence_bits: i64,
        machine_id: i64,
        timestamp: i64,
    ) -> Self {
        let max: i64 = -1;

        let time_mask = !(max << time_bits);
        let machine_mask = !(max << machine_bits);
        let sequence_mask = !(max << sequence_bits);

        let epoch = UNIX_EPOCH.add(Duration::from_nanos(timestamp as u64));

        FastIdWorker {
            time_bits,
            machine_bits,
            sequence_bits,

            time_mask,
            machine_mask,
            sequence_mask,

            machine_id,
            sequence: 0,

            epoch: epoch,

            last_timestamp: 0,
        }
    }

    fn get_current_timestamp(&mut self) -> i64 {
        let duration = SystemTime::now()
            .duration_since(self.epoch)
            .unwrap_or(Duration::new(0, 0));

        let timestamp = duration.as_nanos() >> 20;

        timestamp as i64
    }

    pub fn next_id(&mut self) -> i64 {
        loop {
            let timestamp = self.get_current_timestamp();

            if timestamp > self.last_timestamp {
                self.last_timestamp = timestamp;
                self.sequence = 0
            } else if self.sequence >= self.sequence_mask {
                continue;
            } else {
                self.sequence += 1;
            }

            let id = ((timestamp & self.time_mask) << (self.machine_bits + self.sequence_bits))
                | ((self.sequence & self.sequence_mask) << self.machine_bits)
                | (self.machine_id & self.machine_mask);

            return id;
        }
    }
}

impl Default for FastIdWorker {
    fn default() -> Self {
        FastIdWorker::new(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        println!("");
        let mut worker = FastIdWorker::default();
        let id = worker.next_id();
        println!("{:#064b}", id);
        println!("{}", id);
    }
}
