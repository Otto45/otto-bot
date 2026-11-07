use clap::ValueEnum;

/// `--service-tier`'s values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ServiceTierArgument {
    Standard,
    Fast,
    Ultrafast,
}
