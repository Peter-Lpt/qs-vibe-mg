use std::fs;
use std::path::Path;

use crate::errors::VabError;

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
            let target = fs::read_link(link)?;
            if target == original {
                return Ok(());
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

/// 获取 symlink 或 junction 的目标路径
pub fn read_link_target(link: &Path) -> Result<std::path::PathBuf, VabError> {
    // 先尝试普通 symlink
    if let Ok(target) = fs::read_link(link) {
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

    #[test]
    fn test_is_symlink_nonexistent() {
        assert!(!is_symlink(Path::new("/nonexistent/path")));
    }
}
