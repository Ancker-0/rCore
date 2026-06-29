use pc_keyboard::KeyCode::T;
use rcore_fs::vfs::*;

use crate::process::{self, PROCESSES};

pub struct ProcfsEntry {
    pub pid: usize,
}

impl INode for ProcfsEntry {
    fn read_at(&self, offset: usize, buf: &mut [u8]) -> Result<usize> {
        todo!()
    }

    fn write_at(&self, offset: usize, buf: &[u8]) -> Result<usize> {
        todo!()
    }

    fn poll(&self) -> Result<PollStatus> {
        todo!()
    }

    fn as_any_ref(&self) -> &dyn core::any::Any {
        todo!()
    }

    fn metadata(&self) -> Result<Metadata> {
        let process = PROCESSES.read();
        // process.get(&self.pid).and_then(|p|);
        Ok(Metadata {
            dev: 0,
            inode: 0,
            size: 0,
            blk_size: 0,
            blocks: 0,
            atime: Timespec { sec: 0, nsec: 0 },
            mtime: Timespec { sec: 0, nsec: 0 },
            ctime: Timespec { sec: 0, nsec: 0 },
            type_: FileType::Dir,
            mode: 0o555,
            nlinks: 0,
            uid: 0,
            gid: 0,
            rdev: 0,
        })
    }
}
