use super::*;

impl SnapshotEffector {
    pub fn save_fd_set_rights(
        ctx: &mut FunctionEnvMut<'_, WasiEnv>,
        fd: Fd,
        fs_rights_base: Rights,
        fs_rights_inheriting: Rights,
    ) -> anyhow::Result<()> {
        Self::save_event(
            ctx,
            SnapshotLog::FileDescriptorSetRights {
                fd,
                fs_rights_base,
                fs_rights_inheriting,
            },
        )
    }

    pub fn apply_fd_set_rights(
        ctx: &mut FunctionEnvMut<'_, WasiEnv>,
        fd: Fd,
        fs_rights_base: Rights,
        fs_rights_inheriting: Rights,
    ) -> anyhow::Result<()> {
        crate::syscalls::fd_fdstat_set_rights_internal(ctx, fd, fs_rights_base, fs_rights_inheriting)
            .map_err(|err| {
                anyhow::format_err!(
                    "snapshot restore error: failed to duplicate file descriptor (fd={}, fs_rights_base={:?}, fs_rights_inheriting={:?}) - {}",
                    fd,
                    fs_rights_base,
                    fs_rights_inheriting,
                    err
                )    
            })?;
        Ok(())
    }
}
