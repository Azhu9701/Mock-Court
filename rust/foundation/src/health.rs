use crate::error::Result;
use crate::storage::HealthStatus;

pub struct HealthChecker;

impl HealthChecker {
    pub fn check(
        sqlite_record_count: usize,
        yaml_record_count: usize,
        soul_files_count: usize,
        registry_entries_count: usize,
    ) -> Result<HealthStatus> {
        let sqlite_ok = true;
        let fs_ok = soul_files_count > 0 || registry_entries_count == 0;

        let status = HealthStatus {
            ok: sqlite_ok && fs_ok,
            sqlite_ok,
            fs_ok,
            yaml_count: yaml_record_count,
            sqlite_record_count,
            soul_files_count,
            registry_entries_count,
        };

        Ok(status)
    }
}
