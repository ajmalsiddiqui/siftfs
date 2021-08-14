use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use fuse::{FileAttr, FileType};
use regex::Regex;
use time::Timespec;

fn make_attr(inode: u64, size: u64, is_dir: bool) -> FileAttr {
    let ts = Timespec::new(0, 0);

    FileAttr {
        ino: inode,
        size: size,
        blocks: 0,
        atime: ts,
        mtime: ts,
        ctime: ts,
        crtime: ts,
        kind: if is_dir {
            FileType::Directory
        } else {
            FileType::RegularFile
        },
        perm: 0o755,
        nlink: 0,
        uid: 0,
        gid: 0,
        rdev: 0,
        flags: 0,
    }
}

/// Represents a file or directory that is a part of the SiftFS filesystem
#[derive(Debug)]
pub enum SiftNode {
    SiftFile {
        name: String,           // Name of the file as rendered by sift
        orignal_name: String,   // Name of the original file as it is in the original FS
        abs_path: PathBuf,      // Actual absolute file path of this file
        sift_directory: String, // Directory in which this file resides as a part of SiftFS
        inode: u64,
        offset: i64,
        attr: FileAttr,
    },
    SiftDirectory {
        name: String,
        inode: u64,
        offset: i64,
        attr: FileAttr,
    },
}

impl SiftNode {
    pub fn get_inode(&self) -> u64 {
        match self {
            SiftNode::SiftFile { inode, .. } | SiftNode::SiftDirectory { inode, .. } => *inode,
        }
    }

    pub fn get_offset(&self) -> i64 {
        match self {
            SiftNode::SiftFile { offset, .. } | SiftNode::SiftDirectory { offset, .. } => *offset,
        }
    }

    pub fn get_attr(&self) -> &FileAttr {
        match self {
            SiftNode::SiftFile { attr, .. } | SiftNode::SiftDirectory { attr, .. } => attr,
        }
    }

    pub fn get_name(&self) -> &str {
        match self {
            SiftNode::SiftFile { name, .. } | SiftNode::SiftDirectory { name, .. } => &name,
        }
    }
}

#[derive(Debug)]
pub struct SiftFilesystem {
    directory: PathBuf,
    file_regex: Regex,
    file_format_string: String,
    file_format_string_args: Vec<usize>,
    max_inode: u64,
    max_offset: i64,
    dir_list: HashSet<String>,
    pub nodes: HashMap<u64, SiftNode>,
}

impl SiftFilesystem {
    pub fn new(
        directory: &str,
        file_regex: &str,
        file_format_string: &str,
        file_format_string_args: &str,
    ) -> SiftFilesystem {
        let fmt_string_args = file_format_string_args
            .split(",")
            .map(|x| x.parse::<usize>().unwrap())
            .collect();

        let mut siftfs = SiftFilesystem {
            directory: PathBuf::from(directory),
            file_regex: Regex::new(file_regex).unwrap(),
            file_format_string: file_format_string.to_string(),
            file_format_string_args: fmt_string_args,
            max_inode: 0,
            max_offset: 0,
            dir_list: HashSet::<String>::new(),
            nodes: HashMap::<u64, SiftNode>::new(),
        };

        siftfs.add_directory(&"/");

        siftfs.build_fs();

        siftfs
    }

    pub fn get_node_by_name(&self, node_name: &str) -> Option<&SiftNode> {
        for (_, node) in &self.nodes {
            match node {
                SiftNode::SiftDirectory { name, .. } | SiftNode::SiftFile { name, .. } => {
                    if name == node_name {
                        return Some(&node);
                    }
                }
            };
        }
        return None;
    }

    pub fn get_files_by_directory(&self, dir_name: &str) -> Vec<&SiftNode> {
        self.nodes
            .iter()
            .map(|(_, node)| node)
            // TODO figure out why assiging this to a variable doesn't work
            .filter(|node| match node {
                SiftNode::SiftDirectory { .. } => false,
                SiftNode::SiftFile { name, .. } => name.starts_with(&dir_name),
            })
            .collect()
    }

    fn build_fs(&mut self) {
        let mut fs_tree = HashMap::<String, Vec<PathBuf>>::new();

        for entry in fs::read_dir(&self.directory).unwrap() {
            let path = match entry {
                Err(e) => {
                    println!("error: {}", e);
                    continue;
                }
                Ok(entry) => entry.path(),
            };

            let filename = path.file_name().unwrap();

            let mut captures = self.file_regex.captures_iter(filename.to_str().unwrap());

            let groups = match captures.next() {
                None => {
                    println!(
                        "failed to match file {} against regex {}",
                        filename.to_str().unwrap(),
                        self.file_regex
                    );
                    continue;
                }
                Some(g) => g,
            };

            // The first group represents the directory
            match fs_tree.get_mut(&groups[1]) {
                None => {
                    fs_tree.insert(groups[1].to_string(), vec![path]);
                }
                Some(file_list) => {
                    file_list.push(path);
                }
            };
        }

        for (dir, files) in fs_tree.iter() {
            self.add_directory(&dir);
            for file in files {
                self.add_file(file);
            }
        }
    }

    fn add_directory(&mut self, name: &str) {
        self.max_inode += 1;
        self.max_offset += 1;

        let new_dir = SiftNode::SiftDirectory {
            name: name.to_string(),
            inode: self.max_inode,
            offset: self.max_offset,
            attr: make_attr(self.max_inode, 0, true),
        };

        self.nodes.insert(self.max_inode, new_dir);
    }

    fn add_file(&mut self, abs_path: &Path) {
        self.max_inode += 1;
        self.max_offset += 1;

        let new_file = match self.make_file(abs_path, self.max_inode, self.max_offset) {
            None => {
                return;
            }
            Some(file) => file,
        };

        self.nodes.insert(self.max_inode, new_file);
    }

    fn make_file(&self, file_path: &Path, inode: u64, offset: i64) -> Option<SiftNode> {
        let filename = file_path.file_name().unwrap();

        let mut captures = self.file_regex.captures_iter(filename.to_str().unwrap());
        let groups = match captures.next() {
            None => {
                println!(
                    "failed to match file {} against regex {}",
                    filename.to_str().unwrap(),
                    self.file_regex
                );
                return None;
            }
            Some(g) => g,
        };

        // Because groups[0] is always the original matched string
        // TODO remove this condition
        assert_eq!(groups.len(), self.file_format_string_args.len() + 1);

        let mut formatted_file_name = self.file_format_string.clone();

        for group_no in &self.file_format_string_args {
            let start_offset = formatted_file_name.find('{').unwrap();
            let end_offset = start_offset + 2; // Since the stuff to replace is '{}', and end offset is a closed range
            // *group_no because the for loop iterates over &usize
            formatted_file_name.replace_range(start_offset..end_offset, &groups[*group_no]);
        }

        Some(SiftNode::SiftFile {
            name: formatted_file_name,
            orignal_name: filename.to_str().unwrap().to_string(),
            abs_path: file_path.clone().to_path_buf(),
            sift_directory: groups[1].to_string(),
            inode,
            offset,
            attr: make_attr(inode, file_path.metadata().unwrap().len(), false),
        })
    }
}
