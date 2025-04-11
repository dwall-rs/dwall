//! Cache cleanup service
//!
//! This module provides functionality for cleaning up and maintaining the cache.

use std::path::Path;
use std::time::{Duration, SystemTime};

use dwall::DWALL_CACHE_DIR;
use tokio::fs;

use super::error::CacheResult;
use super::fs_service::FsService;

// 缓存配置常量
const MAX_CACHE_SIZE_BYTES: u64 = 100 * 1024 * 1024; // 100MB
const CACHE_EXPIRY_DAYS: u64 = 30; // 30天过期

/// Service for cleaning up and maintaining the cache
pub struct CleanupService;

impl CleanupService {
    /// Clean up expired cache files
    pub async fn cleanup_expired_cache() -> CacheResult<u64> {
        let thumbnails_dir = DWALL_CACHE_DIR.join("thumbnails");
        if !thumbnails_dir.exists() {
            return Ok(0);
        }

        let now = SystemTime::now();
        let expiry_duration = Duration::from_secs(CACHE_EXPIRY_DAYS * 24 * 60 * 60);
        let mut cleaned_bytes = 0;

        // 遍历缓存目录
        let mut entries = fs::read_dir(&thumbnails_dir).await.map_err(|e| {
            error!(
                dir = %thumbnails_dir.display(),
                error = ?e,
                "Failed to read thumbnails directory"
            );
            e
        })?;

        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();

            if path.is_dir() {
                // 处理主题目录
                let mut theme_entries = match fs::read_dir(&path).await {
                    Ok(entries) => entries,
                    Err(e) => {
                        error!(
                            dir = %path.display(),
                            error = ?e,
                            "Failed to read theme directory"
                        );
                        continue;
                    }
                };

                while let Ok(Some(file_entry)) = theme_entries.next_entry().await {
                    let file_path = file_entry.path();

                    if file_path.is_file() {
                        // 检查文件修改时间
                        if let Ok(metadata) = fs::metadata(&file_path).await {
                            if let Ok(modified) = metadata.modified() {
                                if now
                                    .duration_since(modified)
                                    .map_or(false, |age| age > expiry_duration)
                                {
                                    // 文件过期，删除
                                    let file_size = metadata.len();
                                    if let Err(e) = fs::remove_file(&file_path).await {
                                        error!(
                                            file = %file_path.display(),
                                            error = ?e,
                                            "Failed to remove expired cache file"
                                        );
                                    } else {
                                        cleaned_bytes += file_size;
                                        debug!(
                                            file = %file_path.display(),
                                            size = file_size,
                                            "Removed expired cache file"
                                        );
                                    }
                                }
                            }
                        }
                    }
                }

                // 如果主题目录为空，删除它
                if FsService::is_directory_empty(&path).await {
                    if let Err(e) = fs::remove_dir(&path).await {
                        error!(
                            dir = %path.display(),
                            error = ?e,
                            "Failed to remove empty theme directory"
                        );
                    } else {
                        debug!(
                            dir = %path.display(),
                            "Removed empty theme directory"
                        );
                    }
                }
            }
        }

        Ok(cleaned_bytes)
    }

    /// Enforce cache size limit by removing oldest files when cache is too large
    pub async fn enforce_cache_size_limit() -> CacheResult<u64> {
        let thumbnails_dir = DWALL_CACHE_DIR.join("thumbnails");
        if !thumbnails_dir.exists() {
            return Ok(0);
        }

        // 获取当前缓存大小
        let current_size = FsService::get_directory_size(&thumbnails_dir).await?;

        // 如果缓存大小未超过限制，不需要清理
        if current_size <= MAX_CACHE_SIZE_BYTES {
            return Ok(0);
        }

        // 收集所有缓存文件信息
        let mut cache_files = Vec::new();
        let mut entries = fs::read_dir(&thumbnails_dir).await.map_err(|e| {
            error!(
                dir = %thumbnails_dir.display(),
                error = ?e,
                "Failed to read thumbnails directory"
            );
            e
        })?;

        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();

            if path.is_dir() {
                let mut theme_entries = match fs::read_dir(&path).await {
                    Ok(entries) => entries,
                    Err(e) => {
                        error!(
                            dir = %path.display(),
                            error = ?e,
                            "Failed to read theme directory"
                        );
                        continue;
                    }
                };

                while let Ok(Some(file_entry)) = theme_entries.next_entry().await {
                    let file_path = file_entry.path();

                    if file_path.is_file() {
                        if let Ok(metadata) = fs::metadata(&file_path).await {
                            if let Ok(modified) = metadata.modified() {
                                cache_files.push((file_path, modified, metadata.len()));
                            }
                        }
                    }
                }
            }
        }

        // 按修改时间排序（最旧的在前面）
        cache_files.sort_by(|a, b| a.1.cmp(&b.1));

        // 计算需要删除的大小
        let target_size = current_size - MAX_CACHE_SIZE_BYTES;
        let mut cleaned_bytes = 0;

        // 从最旧的文件开始删除，直到达到目标大小
        for (file_path, _, file_size) in cache_files {
            if cleaned_bytes >= target_size {
                break;
            }

            if let Err(e) = fs::remove_file(&file_path).await {
                error!(
                    file = %file_path.display(),
                    error = ?e,
                    "Failed to remove cache file during size enforcement"
                );
            } else {
                cleaned_bytes += file_size;
                debug!(
                    file = %file_path.display(),
                    size = file_size,
                    "Removed cache file to enforce size limit"
                );
            }
        }

        // 清理空目录
        Self::clean_empty_directories(&thumbnails_dir).await?;

        Ok(cleaned_bytes)
    }

    /// Clean empty directories recursively
    pub async fn clean_empty_directories(dir: &Path) -> CacheResult<()> {
        let mut entries = fs::read_dir(dir).await.map_err(|e| {
            error!(
                dir = %dir.display(),
                error = ?e,
                "Failed to read directory"
            );
            e
        })?;

        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();

            if path.is_dir() {
                Box::pin(Self::clean_empty_directories(&path)).await?;

                if FsService::is_directory_empty(&path).await {
                    if let Err(e) = fs::remove_dir(&path).await {
                        error!(
                            dir = %path.display(),
                            error = ?e,
                            "Failed to remove empty directory"
                        );
                    } else {
                        debug!(
                            dir = %path.display(),
                            "Removed empty directory"
                        );
                    }
                }
            }
        }

        Ok(())
    }
}
