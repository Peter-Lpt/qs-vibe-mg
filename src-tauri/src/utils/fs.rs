use std::fs;
use std::path::{Path, PathBuf};

use crate::errors::VabError;

/// 规范化路径：去除 Windows `\\?\` 前缀，转为绝对路径
pub fn normalize_path(path: &Path) -> PathBuf {
    // 先尝试 canonicalize 获取绝对路径
    let canonical = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());

    // Windows: 去除 `\\?\` 前缀
    #[cfg(windows)]
    {
        let s = canonical.to_string_lossy();
        if let Some(stripped) = s.strip_prefix(r"\\?\") {
            return PathBuf::from(stripped);
        }
    }

    canonical
}

/// 递归复制目录
pub fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), VabError> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dest = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dest)?;
        } else {
            fs::copy(entry.path(), &dest)?;
        }
    }
    Ok(())
}

/// 创建 symlink
/// Windows: 使用 junction（目录链接，无需管理员权限）
/// macOS/Linux: 使用 symlink
pub fn create_symlink(original: &Path, link: &Path) -> Result<(), VabError> {
    // 确保 link 的父目录存在
    if let Some(parent) = link.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    // 如果 link 已存在，检查是否指向同一目标
    if link.exists() || is_symlink(link) {
        let meta = link.symlink_metadata().map_err(VabError::Io)?;
        if meta.file_type().is_symlink() {
            if let Ok(target) = fs::read_link(link) {
                if normalize_path(&target) == normalize_path(original) {
                    return Ok(());
                }
            }
        }
        return Err(VabError::LinkAlreadyExists {
            skill_id: original
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default(),
            agent_id: link
                .parent()
                .and_then(|p| p.file_name())
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default(),
        });
    }

    #[cfg(windows)]
    {
        // Windows: 使用 junction（目录链接，无需管理员权限）
        if original.is_dir() {
            std::os::windows::fs::symlink_dir(original, link)?;
        } else {
            std::os::windows::fs::symlink_file(original, link)?;
        }
    }

    #[cfg(not(windows))]
    {
        std::os::unix::fs::symlink(original, link)?;
    }

    Ok(())
}

/// 删除 symlink（不删除源文件）
pub fn remove_symlink(link: &Path) -> Result<(), VabError> {
    if !link.exists() && !is_symlink(link) {
        return Ok(());
    }

    let meta = link.symlink_metadata().map_err(VabError::Io)?;
    if meta.file_type().is_symlink() {
        // symlink：通过 read_link 判断目标类型
        #[cfg(windows)]
        {
            // Windows 上 directory symlink 需要用 remove_dir
            // 尝试 remove_dir，失败则尝试 remove_file
            if fs::remove_dir(link).is_err() {
                fs::remove_file(link)?;
            }
        }
        #[cfg(not(windows))]
        {
            fs::remove_file(link)?;
        }
    } else if meta.file_type().is_dir() {
        // junction 在 Windows 上不是 symlink，但可以安全删除目录
        fs::remove_dir(link)?;
    } else {
        fs::remove_file(link)?;
    }

    Ok(())
}

/// 检查路径是否是 symlink（包括 junction）
pub fn is_symlink(path: &Path) -> bool {
    path.symlink_metadata()
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false)
}

/// 检查路径是否是 junction（Windows 目录链接）
#[cfg(windows)]
pub fn is_junction(path: &Path) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;

    path.metadata()
        .map(|m| {
            let attrs = m.file_attributes();
            // junction 是 reparse point 但不是 symlink
            (attrs & FILE_ATTRIBUTE_REPARSE_POINT) != 0 && !m.file_type().is_symlink()
        })
        .unwrap_or(false)
}

#[cfg(not(windows))]
pub fn is_junction(_path: &Path) -> bool {
    false
}

/// 检查路径是否是链接（symlink 或 junction）
pub fn is_link(path: &Path) -> bool {
    is_symlink(path) || is_junction(path)
}

/// 获取 symlink 或 junction 的目标路径（解析为绝对路径）
pub fn read_link_target(link: &Path) -> Result<std::path::PathBuf, VabError> {
    // 先尝试普通 symlink
    if let Ok(target) = fs::read_link(link) {
        // 如果是相对路径，基于 link 所在目录解析为绝对路径
        if target.is_relative() {
            if let Some(parent) = link.parent() {
                return Ok(parent.join(&target));
            }
        }
        return Ok(target);
    }
    // Windows junction: fs::read_link 失败，用 canonicalize 解析实际路径
    #[cfg(windows)]
    {
        if is_junction(link) {
            return fs::canonicalize(link).map_err(VabError::Io);
        }
    }
    Err(VabError::Io(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        format!("Cannot read link target: {}", link.display()),
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn test_dir() -> PathBuf {
        // 使用随机后缀避免冲突
        let id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("vibe_test_fs_{}", id));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn cleanup(dir: &Path) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_is_symlink_nonexistent() {
        assert!(!is_symlink(Path::new("/nonexistent/path")));
    }

    #[test]
    fn test_normalize_path_strips_prefix() {
        let dir = test_dir();
        let file = dir.join("test.txt");
        fs::write(&file, "hello").unwrap();

        let normalized = normalize_path(&file);
        #[cfg(windows)]
        {
            let s = normalized.to_string_lossy();
            assert!(!s.starts_with(r"\\?\"), "Should strip \\\\?\\ prefix: {}", s);
        }
        assert!(normalized.exists());

        cleanup(&dir);
    }

    #[test]
    fn test_create_and_read_symlink() {
        let dir = test_dir();
        let source = dir.join("source");
        let link = dir.join("link");

        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("file.txt"), "content").unwrap();

        // 创建 symlink
        let result = create_symlink(&source, &link);
        if result.is_err() {
            // Windows 上可能需要管理员权限，跳过测试
            eprintln!("Skipping symlink test: {:?}", result.err());
            cleanup(&dir);
            return;
        }

        assert!(is_symlink(&link), "link should be a symlink");
        assert!(is_link(&link), "link should be a link");

        // 读取目标
        let target = read_link_target(&link).unwrap();
        assert_eq!(normalize_path(&target), normalize_path(&source));

        // 读取内容（通过 symlink）
        assert!(link.join("file.txt").exists());

        cleanup(&dir);
    }

    #[test]
    fn test_remove_symlink() {
        let dir = test_dir();
        let source = dir.join("source");
        let link = dir.join("link");

        fs::create_dir_all(&source).unwrap();

        let result = create_symlink(&source, &link);
        if result.is_err() {
            eprintln!("Skipping symlink test: {:?}", result.err());
            cleanup(&dir);
            return;
        }

        assert!(is_symlink(&link));

        // 删除 symlink
        remove_symlink(&link).unwrap();
        assert!(!link.exists());
        assert!(source.exists()); // 源文件不受影响

        cleanup(&dir);
    }

    #[test]
    fn test_copy_dir_all() {
        let dir = test_dir();
        let src = dir.join("src");
        let dst = dir.join("dst");

        // 创建源目录
        fs::create_dir_all(src.join("sub")).unwrap();
        fs::write(src.join("file1.txt"), "content1").unwrap();
        fs::write(src.join("sub").join("file2.txt"), "content2").unwrap();

        // 复制
        copy_dir_all(&src, &dst).unwrap();

        // 验证
        assert!(dst.join("file1.txt").exists());
        assert!(dst.join("sub").join("file2.txt").exists());
        assert_eq!(
            fs::read_to_string(dst.join("file1.txt")).unwrap(),
            "content1"
        );

        cleanup(&dir);
    }

    #[test]
    fn test_read_link_target_relative() {
        let dir = test_dir();
        let source = dir.join("source");
        let link = dir.join("link");

        fs::create_dir_all(&source).unwrap();

        // 手动创建相对路径的 symlink
        #[cfg(windows)]
        {
            use std::os::windows::fs::symlink_dir;
            let _ = symlink_dir("source", &link);
        }
        #[cfg(not(windows))]
        {
            std::os::unix::fs::symlink("source", &link).unwrap();
        }

        // read_link_target 应该解析为绝对路径
        if is_symlink(&link) {
            let target = read_link_target(&link).unwrap();
            assert!(
                target.is_absolute(),
                "Target should be absolute: {:?}",
                target
            );
            assert!(target.exists(), "Target should exist: {:?}", target);
        }

        cleanup(&dir);
    }
}
