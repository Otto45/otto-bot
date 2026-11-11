use crate::managers::data_layer::managers::gateways::relational_database::resource_models::advisory_lock_row::AdvisoryLockRow;

pub struct AdvisoryLockMapper;

impl AdvisoryLockMapper {
    /// Whether the lock was taken. Expects a row that has already passed ValidatorAdvisoryLockRow: the `expect` states
    /// a checked fact, and fails only if the validator is wrong.
    pub fn to_locked(advisory_lock_row: AdvisoryLockRow) -> bool {
        advisory_lock_row.locked.expect("validated by ValidatorAdvisoryLockRow")
    }
}
