use argon2::Params;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProfileId {
    V1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Profile {
    id: ProfileId,
    domain: &'static str,
    memory_cost_kib: u32,
    time_cost: u32,
    parallelism: u32,
    output_len: usize,
}

impl Profile {
    pub const fn v1() -> Self {
        Self {
            id: ProfileId::V1,
            domain: "mind-wallet-monero-v1",
            memory_cost_kib: 19 * 1024,
            time_cost: 2,
            parallelism: 1,
            output_len: 32,
        }
    }

    pub const fn id(&self) -> ProfileId {
        self.id
    }

    pub const fn domain(&self) -> &'static str {
        self.domain
    }

    pub const fn memory_cost_kib(&self) -> u32 {
        self.memory_cost_kib
    }

    pub const fn time_cost(&self) -> u32 {
        self.time_cost
    }

    pub const fn parallelism(&self) -> u32 {
        self.parallelism
    }

    pub const fn output_len(&self) -> usize {
        self.output_len
    }

    pub fn argon2_params(&self) -> argon2::Result<Params> {
        Params::new(
            self.memory_cost_kib,
            self.time_cost,
            self.parallelism,
            Some(self.output_len),
        )
    }
}
