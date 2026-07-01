use core::ops::Deref;

use alloc::{
    string::{String, ToString},
    sync::{Arc, Weak},
};
use rcore_fs::vfs::*;

use crate::process::{PROCESSES, THREADS, current_thread};
mod entry;

pub struct Procfs;

impl Procfs {
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl FileSystem for Procfs {
    fn sync(&self) -> Result<()> {
        todo!()
    }

    fn root_inode(&self) -> Arc<dyn INode> {
        struct ProcRoot {}
        impl INode for ProcRoot {
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
            fn metadata(&self) -> Result<Metadata> {
                Ok(Metadata {
                    dev: 0,
                    inode: 1,
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
            fn create(&self, name: &str, type_: FileType, mode: u32) -> Result<Arc<dyn INode>> {
                Err(FsError::NotSupported)
            }
            fn get_entry(&self, id: usize) -> Result<String> {
                match id {
                    0 => Ok(String::from(".")),
                    1 => Ok(String::from("..")),
                    2 => Ok(String::from("self")),
                    i => {
                        let process = PROCESSES.read();
                        let my_pid = current_thread().unwrap().proc.lock().pid.0;
                        // for j in process.keys() {
                        //     info!("HAHA {:?}\n", process.keys().nth(i - 2));
                        // }
                        process
                            .iter()
                            .filter(|(_, p)| !p.lock().hidden)
                            .nth(i - 3)
                            .map(|(pid, _)| pid.to_string())
                            .ok_or(FsError::EntryNotFound)
                    }
                }
            }
            fn find(&self, name: &str) -> Result<Arc<dyn INode>> {
                match name {
                    "." | ".." => Ok(Procfs::new().root_inode()),
                    "self" => {
                        let thread = current_thread().unwrap();
                        Ok(Arc::new(entry::ProcfsEntryDir { proc: Arc::downgrade(&thread.proc) }))
                    }
                    name => {
                        let process = PROCESSES.read();
                        name.parse::<usize>()
                            .ok()
                            .and_then(|pid| {
                                process.get(&pid).and_then(|p| match p.lock().hidden {
                                    false => {
                                        Some(Arc::new(entry::ProcfsEntryDir { proc: Arc::downgrade(p) })
                                            as Arc<dyn INode>)
                                    }
                                    true => None,
                                })
                            })
                            .ok_or(FsError::EntryNotFound)
                    }
                }
            }
            fn fs(&self) -> Arc<dyn FileSystem> {
                Arc::new(Procfs {})
            }
        }
        Arc::new(ProcRoot {})
    }

    fn info(&self) -> FsInfo {
        todo!()
    }
}
