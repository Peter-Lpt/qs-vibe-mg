use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
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
        let name_str = name.to_string_lossy();
        if is_hash_ignored_name(&name_str) {
            continue;
        }

        if path.is_dir() {
            hasher.update(b"dir:");
            hasher.update(name_str.as_bytes());
            hasher.update(b"\n");
            hash_dir_recursive(&path, hasher);
        } else if let Ok(content) = fs::read(&path) {
            hasher.update(b"file:");
            hasher.update(name_str.as_bytes());
            hasher.update(b":");
            hasher.update(&content);
            hasher.update(b"\n");
        }
    }
}

// ── P1：哈希缓存 ───────────────────────────────────────────────────────
// 缓存键为 (mtime_ns, size_bytes, file_count) 三元组；三元组全相等则复用缓存中
// 已存的真 SHA-256，不再读取文件内容。对外返回的始终是真哈希——content_hash 被前端
// useSkillAgentStatus.ts 用于冲突判定，绝不能退化成指纹（见 10 性能优化方案 P1）。

const CACHE_FILE: &str = ".vibe-hash-cache.json";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HashCache {
    /// 目录路径 -> (mtime_ns, 总字节数, 文件数, sha256)
    pub entries: std::collections::HashMap<String, (u64, u64, u64, String)>,
}

/// 从 vibe 目录加载哈希缓存（文件缺失/损坏时返回空缓存，不影响正确性）
pub fn load_hash_cache(vibe_dir: &Path) -> HashCache {
    let path = vibe_dir.join(CACHE_FILE);
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str::<HashCache>(&s).ok())
        .unwrap_or_default()
}

/// 原子写回缓存：写入临时文件后 rename，避免并发/中断损坏缓存文件
pub fn save_hash_cache(vibe_dir: &Path, cache: &HashCache) {
    let path = vibe_dir.join(CACHE_FILE);
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            let _ = fs::create_dir_all(parent);
        }
    }
    let Ok(content) = serde_json::to_string(cache) else {
        return;
    };
    let tmp = vibe_dir.join(format!("{}.tmp", CACHE_FILE));
    if fs::write(&tmp, &content).is_ok() {
        let _ = fs::rename(&tmp, &path);
    }
}

/// 聚合目录的 (mtime_ns, 总字节数, 文件数)，作为缓存失效键。
/// 仅读取元数据，不读取文件内容，成本远低于递归 SHA-256。
fn dir_meta(dir: &Path) -> (u64, u64, u64) {
    let mut mtime_ns = 0u64;
    let mut size = 0u64;
    let mut count = 0u64;

    fn walk(dir: &Path, mtime_ns: &mut u64, size: &mut u64, count: &mut u64) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if is_hash_ignored_name(&name_str) {
                    continue;
                }
                if let Ok(m) = entry.metadata() {
                    if m.is_dir() {
                        walk(&p, mtime_ns, size, count);
                    } else {
                        *count += 1;
                        *size += m.len();
                        if let Ok(modified) = m.modified() {
                            let ns = modified
                                .duration_since(std::time::UNIX_EPOCH)
                                .map(|d| d.as_nanos() as u64)
                                .unwrap_or(0);
                            if ns > *mtime_ns {
                                *mtime_ns = ns;
                            }
                        }
                    }
                }
            }
        }
    }

    walk(dir, &mut mtime_ns, &mut size, &mut count);
    (mtime_ns, size, count)
}

/// 从缓存取哈希；未命中则计算真哈希并写回缓存。调用方负责在结束时 save_hash_cache。
pub fn dir_hash_into(cache: &mut HashCache, dir: &Path) -> String {
    if !dir.exists() {
        return String::new();
    }
    let (mtime_ns, size, count) = dir_meta(dir);
    let key = dir.to_string_lossy().to_string();
    if let Some((cm, cs, cc, hash)) = cache.entries.get(&key) {
        if *cm == mtime_ns && *cs == size && *cc == count {
            return hash.clone();
        }
    }
    let hash = dir_hash(dir);
    cache.entries.insert(key, (mtime_ns, size, count, hash.clone()));
    hash
}

fn is_hash_ignored_name(name: &str) -> bool {
    matches!(
        name,
        ".git" | ".vibe-origin.json" | ".vibe-origin" | ".vibe-hash-cache.json"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn make_dir() -> std::path::PathBuf {
        let base = std::env::temp_dir().join(format!(
            "qs_hash_test_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).unwrap();
        let f = base.join("SKILL.md");
        let mut w = std::fs::File::create(&f).unwrap();
        w.write_all(b"name: x\ndescription: y\n").unwrap();
        base
    }

    #[test]
    fn cache_reuses_hash_when_unchanged() {
        let dir = make_dir();
        let mut cache = HashCache::default();
        let h1 = dir_hash_into(&mut cache, &dir);
        // 第二次：三元组未变，应返回相同哈希且不再新增条目
        let h2 = dir_hash_into(&mut cache, &dir);
        assert_eq!(h1, h2);
        assert_eq!(cache.entries.len(), 1);

        // 修改内容后三元组变化（mtime/size），应计算新哈希
        std::thread::sleep(std::time::Duration::from_millis(1100));
        let f = dir.join("SKILL.md");
        let mut w = std::fs::File::create(&f).unwrap();
        w.write_all(b"name: x\ndescription: changed\n").unwrap();
        let h3 = dir_hash_into(&mut cache, &dir);
        assert_ne!(h1, h3);
        assert_eq!(cache.entries.len(), 1);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn hash_ignores_git_metadata() {
        let dir = make_dir();
        std::fs::create_dir_all(dir.join(".git")).unwrap();
        std::fs::write(dir.join(".git").join("HEAD"), b"ref: refs/heads/main").unwrap();

        let h1 = dir_hash(&dir);
        std::fs::write(dir.join(".git").join("HEAD"), b"ref: refs/heads/dev").unwrap();
        let h2 = dir_hash(&dir);

        assert_eq!(h1, h2);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
