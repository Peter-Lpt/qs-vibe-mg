use std::fs;
use std::path::Path;

use sha2::{Digest, Sha256};

/// 计算目录内容的 SHA-256 hex 哈希（递归哈希所有文件，用于比较 skill 内容是否一致）
pub fn dir_hash(dir: &Path) -> String {
    if !dir.exists() {
        return String::new();
    }
    let mut hasher = Sha256::new();
    hash_dir_recursive(dir, &mut hasher);
    format!("{:x}", hasher.finalize())
}

fn hash_dir_recursive(dir: &Path, hasher: &mut Sha256) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    let mut sorted: Vec<_> = entries.filter_map(|e| e.ok()).collect();
    sorted.sort_by_key(|e| e.file_name());

    for entry in sorted {
        let path = entry.path();
        let name = entry.file_name();

        if path.is_dir() {
            hasher.update(b"dir:");
            let name_str = name.to_string_lossy();
            hasher.update(name_str.as_bytes());
            hasher.update(b"\n");
            hash_dir_recursive(&path, hasher);
        } else if let Ok(content) = fs::read(&path) {
            hasher.update(b"file:");
            let name_str = name.to_string_lossy();
            hasher.update(name_str.as_bytes());
            hasher.update(b":");
            hasher.update(&content);
            hasher.update(b"\n");
        }
    }
}
