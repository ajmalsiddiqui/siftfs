use std::ffi::OsStr;
use std::fs;
use std::path::Path;

use fuse::{FileType, Filesystem, ReplyAttr, ReplyData, ReplyDirectory, ReplyEntry, Request};
use libc::ENOENT;
use time::Timespec;

use crate::sift::{SiftFilesystem, SiftNode};

impl Filesystem for SiftFilesystem {
    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        println!("getattr(ino={})", ino);

        match self.nodes.get(&ino) {
            None => reply.error(ENOENT),
            Some(node) => {
                let ttl = Timespec::new(1, 0);
                reply.attr(&ttl, node.get_attr());
            }
        };

        return;
    }

    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        let name = name.to_os_string().into_string().unwrap();
        println!("lookup(parent={}, name={})", parent, name);

        match self.get_node_by_name(&name) {
            None => reply.error(ENOENT),
            Some(node) => {
                let ttl = Timespec::new(1, 0);
                reply.entry(&ttl, node.get_attr(), 0);
            }
        };

        return;
    }

    fn readdir(
        &mut self,
        _req: &Request,
        ino: u64,
        fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        println!("readdir(ino={}, fh={}, offset={})", ino, fh, offset);
        if offset == 0 {
            if ino == 1 {
                for (_, file) in self
                    .nodes
                    .iter()
                    .filter(|(&inode, _)| inode > 1)
                    .filter(|(_, node)| matches!(node, SiftNode::SiftDirectory { .. }))
                {
                    reply.add(
                        file.get_inode(),
                        file.get_offset(),
                        FileType::Directory,
                        &Path::new(file.get_name()),
                    );
                }
                reply.ok();
                return;
            } else {
                match self.nodes.get(&ino) {
                    None => {
                        reply.error(ENOENT);
                        return;
                    }
                    Some(possible_dir) => match possible_dir {
                        SiftNode::SiftFile { .. } => {
                            reply.error(ENOENT);
                            return;
                        }
                        SiftNode::SiftDirectory { .. } => {
                            for file in self.get_files_by_directory(possible_dir.get_name()) {
                                reply.add(
                                    file.get_inode(),
                                    file.get_offset(),
                                    FileType::RegularFile,
                                    &Path::new(file.get_name()),
                                );
                            }
                            reply.ok();
                            return;
                        }
                    },
                }
            }
        }
        reply.error(ENOENT);
    }

    fn read(
        &mut self,
        _req: &Request,
        ino: u64,
        fh: u64,
        offset: i64,
        size: u32,
        reply: ReplyData,
    ) {
        println!(
            "read(ino={}, fh={}, offset={}, size={})",
            ino, fh, offset, size
        );

        match self.nodes.get(&ino) {
            None => reply.error(ENOENT),
            Some(node) => match node {
                SiftNode::SiftDirectory { .. } => reply.error(ENOENT),
                SiftNode::SiftFile { abs_path, .. } => {
                    let data_as_bytes = fs::read(abs_path).unwrap();
                    let byte_count: usize =
                        std::cmp::min(offset as usize + size as usize, data_as_bytes.len());
                    reply.data(&data_as_bytes[offset as usize..byte_count]);
                }
            },
        }

        return;
    }
}
