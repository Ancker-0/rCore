use crate::{fs::Pseudo, sync::SpinNoIrqLock as Mutex};
use alloc::{
    string::{String, ToString},
    sync::Arc,
};
use pc_keyboard::KeyCode::T;
use rcore_fs::vfs::{FileType::File, *};

use crate::{
    arch::cpu::id,
    fs::Procfs,
    process::{self, Process, PROCESSES},
};

pub struct ProcfsEntryDir {
    pub pid: usize,
}

impl INode for ProcfsEntryDir {
    fn read_at(&self, offset: usize, buf: &mut [u8]) -> Result<usize> {
        Err(FsError::IsDir)
    }

    fn write_at(&self, offset: usize, buf: &[u8]) -> Result<usize> {
        Err(FsError::IsDir)
    }

    fn poll(&self) -> Result<PollStatus> {
        Err(FsError::IsDir)
    }

    fn as_any_ref(&self) -> &dyn core::any::Any {
        self
    }

    fn create(&self, name: &str, type_: FileType, mode: u32) -> Result<Arc<dyn INode>> {
        Err(FsError::NotSupported)
    }

    fn metadata(&self) -> Result<Metadata> {
        // let process = PROCESSES.read();
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
            nlinks: 2,
            uid: 0,
            gid: 0,
            rdev: 0,
        })
    }

    fn get_entry(&self, id: usize) -> Result<String> {
        match id {
            0 => Ok(String::from(".")),
            1 => Ok(String::from("..")),
            i => {
                let process = PROCESSES.read();
                let p = process.get(&self.pid);
                if let Some(p) = p {
                    if p.lock().hidden {
                        Err(FsError::DirRemoved)
                    } else {
                        PROC_ENTRIES
                            .get(i - 2)
                            .map(|entry| entry.name.clone())
                            .ok_or(FsError::DirRemoved)
                    }
                } else {
                    Err(FsError::EntryNotFound)
                }
            }
        }
    }
    fn find(&self, name: &str) -> Result<Arc<dyn INode>> {
        match name {
            "." | ".." => Ok(Procfs::new().root_inode()),
            name => {
                let process = PROCESSES.read();
                let p = process.get(&self.pid);
                if let Some(p) = p {
                    if let Some(entry_id) = PROC_ENTRIES.iter().position(|entry| entry.name == name)
                    {
                        Ok(Arc::new(ProcfsEntry {
                            pid: self.pid,
                            entry_id,
                        }))
                    } else {
                        Err(FsError::EntryNotFound)
                    }
                } else {
                    Err(FsError::EntryNotFound)
                }
            }
        }
    }
}

pub struct ProcfsEntry {
    pub pid: usize,
    entry_id: usize,
}

impl INode for ProcfsEntry {
    fn read_at(&self, offset: usize, buf: &mut [u8]) -> Result<usize> {
        let process = PROCESSES.read();
        let p = process.get(&self.pid);
        if let Some(p) = p {
            PROC_ENTRIES
                .get(self.entry_id)
                .map(|entry| (entry.func)(p.clone()))
                .ok_or(FsError::EntryNotFound)?
                .read_at(offset, buf)
        } else {
            Err(FsError::EntryNotFound)
        }
    }
    fn write_at(&self, _offset: usize, _buf: &[u8]) -> Result<usize> {
        Err(FsError::NotSupported)
    }
    fn poll(&self) -> Result<PollStatus> {
        Ok(PollStatus {
            read: true,
            write: false,
            error: false,
        })
    }
    fn metadata(&self) -> Result<Metadata> {
        Ok(Metadata {
            dev: 0,
            inode: 0,
            size: 0,
            blk_size: 0,
            blocks: 0,
            atime: Timespec { sec: 0, nsec: 0 },
            mtime: Timespec { sec: 0, nsec: 0 },
            ctime: Timespec { sec: 0, nsec: 0 },
            type_: FileType::File,
            mode: 0,
            nlinks: 1,
            uid: 0,
            gid: 0,
            rdev: 0,
        })
    }
    fn as_any_ref(&self) -> &dyn core::any::Any {
        self
    }
}

lazy_static! {
    pub static ref PROC_ENTRIES: [ProcEntries; 2] = [
        ProcEntries {
            name: "status".to_string(),
            func: |process| Arc::new(Pseudo::new("1", FileType::File)),
        },
        ProcEntries {
            name: "parent".to_string(),
            func: |process| {
                let parent_pid = process.lock().parent.0;
                Arc::new(Pseudo::new(&parent_pid.to_string(), FileType::File))
            },
        }
    ];
}

pub struct ProcEntries {
    name: String,
    func: fn(Arc<Mutex<Process>>) -> Arc<dyn INode>,
}
