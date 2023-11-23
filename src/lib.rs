use std::ops::Add;
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const DEFAULT_EPOCH: u64 = 1527811200000000000;

pub struct FastId(i64, #[cfg(feature = "guid")] uuid::Uuid);

impl FastId {
    pub fn as_i64(&self) -> i64 {
        self.0
    }

    pub fn as_u64(&self) -> u64 {
        self.0 as u64
    }

    #[cfg(feature = "guid")]
    pub fn as_guid(&self) -> uuid::Uuid {
        self.1
    }

    #[cfg(feature = "base62")]
    pub fn to_base62(&self) -> String {
        format!("{:0>11}", base62::encode(self.as_u64()))
    }

    #[cfg(feature = "base64")]
    pub fn to_base64(&self) -> String {
        use base64::{engine::general_purpose::STANDARD, Engine as _};

        let bytes = u64::to_le_bytes(self.as_u64());
        format!("{:0>12}", STANDARD.encode(bytes))
    }
}

impl std::fmt::Binary for FastId {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.0.fmt(fmt)
    }
}

impl std::fmt::Display for FastId {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.0.fmt(fmt)
    }
}

#[derive(Debug)]
pub struct FastIdWorker {
    time_bits: usize,
    machine_bits: usize,
    sequence_bits: usize,
    #[cfg(feature = "guid")]
    placeholder_bits: usize,

    time_mask: u64,
    machine_mask: u64,
    sequence_mask: u64,
    #[cfg(feature = "guid")]
    placeholder_mask: u64,

    machine_id: u64,
    sequence: Mutex<u64>,

    epoch: SystemTime,

    last_timestamp: Mutex<u64>,
}

impl FastIdWorker {
    pub fn new(machine_id: u64) -> Self {
        // time_bits: 40,
        // machine_bits: 16,
        // sequence_bits: 7,

        FastIdWorker::with_bits(40, 16, 7, machine_id)
    }

    pub fn with_bits(
        time_bits: usize,
        machine_bits: usize,
        sequence_bits: usize,
        machine_id: u64,
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
        time_bits: usize,
        machine_bits: usize,
        sequence_bits: usize,
        machine_id: u64,
        timestamp: u64,
    ) -> Self {
        let max = u64::MAX;

        let time_mask = !(max << time_bits);
        let machine_mask = !(max << machine_bits);
        let sequence_mask = !(max << sequence_bits);

        #[cfg(feature = "guid")]
        let placeholder_bits = 14 - sequence_bits;
        #[cfg(feature = "guid")]
        let placeholder_mask = !(max << placeholder_bits);

        let epoch = UNIX_EPOCH.add(Duration::from_nanos(timestamp));

        FastIdWorker {
            time_bits,
            machine_bits,
            sequence_bits,
            #[cfg(feature = "guid")]
            placeholder_bits,

            time_mask,
            machine_mask,
            sequence_mask,
            #[cfg(feature = "guid")]
            placeholder_mask,

            machine_id,
            sequence: Mutex::new(0),

            epoch: epoch,

            last_timestamp: Mutex::new(0),
        }
    }

    fn get_current_timestamp(&self) -> u64 {
        let duration = SystemTime::now()
            .duration_since(self.epoch)
            .unwrap_or(Duration::new(0, 0));

        let timestamp = duration.as_nanos() >> 20;

        timestamp as u64
    }

    pub fn next_id(&self) -> FastId {
        loop {
            let ts = self.get_current_timestamp();

            let mut last_timestamp = self.last_timestamp.lock().unwrap();
            let mut sequence = self.sequence.lock().unwrap();

            if ts > *last_timestamp {
                *last_timestamp = ts;
                *sequence = 0
            } else if *sequence >= self.sequence_mask {
                continue;
            } else {
                *sequence += 1;
            }

            let id = ((ts & self.time_mask) << (self.machine_bits + self.sequence_bits))
                | ((*sequence & self.sequence_mask) << self.machine_bits)
                | (self.machine_id & self.machine_mask);
            let id = id as i64;

            #[cfg(feature = "guid")]
            {
                // codes from https://github.com/uuid-rs/uuid/blob/805f4edd4d356dc05b5be55397f7fb43e47a78eb/src/v1.rs#L195-L216

                let time_low = (ts & 0xFFFF_FFFF) as u32;
                let time_mid = ((ts >> 32) & 0xFFFF) as u16;
                let time_high_and_version = (((ts >> 48) & 0x0FFF) as u16) | (1 << 12);

                let mut d4 = [0; 8];

                let sequence = (*sequence << self.placeholder_bits) | (ts & self.placeholder_mask);

                d4[0] = (((sequence & 0x3F00) >> 8) as u8) | 0x80;
                d4[1] = (sequence & 0xFF) as u8;

                let node_id = u64::to_be_bytes(self.machine_id & 0xFFFF_FFFF_FFFF);
                d4[2..].copy_from_slice(&node_id[2..]);

                let guid = uuid::Uuid::from_fields(time_low, time_mid, time_high_and_version, &d4);

                return FastId(id, guid);
            }

            #[cfg(not(feature = "guid"))]
            return FastId(id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_generate_id() {
        let mut worker = FastIdWorker::new(u64::MAX);
        let id = worker.next_id();

        assert_eq!(format!("{:#064b}", id), format!("{:#064b}", id.as_i64()));
        assert_eq!(format!("{}", id), format!("{}", id.as_i64()));
    }

    #[test]
    fn can_generate_many_ids() {
        let mut worker = FastIdWorker::new(u64::MAX);

        let mut last_id = worker.next_id();
        for _ in 0..1000 {
            let id = worker.next_id();
            assert!(id.as_i64() > last_id.as_i64());
            last_id = id;
        }
    }
}
