use std::path::{Path, PathBuf};

use tokio::fs;

#[derive(Debug, Clone)]
pub struct ApkCandidate {
    pub path: PathBuf,
    pub filename: String,
    pub version: Option<(u64, u64, u64)>,
}

fn parse_version_from_filename(filename: &str) -> Option<(u64, u64, u64)> {
    // Ожидаемые форматы:
    // - app-release-v1.0.56.apk
    // - app-release-1.0.56.apk
    // - v1.0.56.apk
    let name = filename.trim();
    let name = name.strip_suffix(".apk")?;

    let raw = name
        .strip_prefix("app-release-v")
        .or_else(|| name.strip_prefix("app-release-"))
        .or_else(|| name.strip_prefix('v'))?;

    let mut parts = raw.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    Some((major, minor, patch))
}

fn choose_best(mut candidates: Vec<ApkCandidate>) -> Option<ApkCandidate> {
    if candidates.is_empty() {
        return None;
    }

    // Предпочитаем версии, которые удалось распарсить
    candidates.sort_by(|a, b| match (a.version, b.version) {
        (Some(av), Some(bv)) => av.cmp(&bv),
        (Some(_), None) => std::cmp::Ordering::Greater,
        (None, Some(_)) => std::cmp::Ordering::Less,
        (None, None) => a.filename.cmp(&b.filename),
    });

    candidates.pop()
}

/// Находит APK для раздачи:
/// - Если `path_or_dir` указывает на файл — возвращает его
/// - Если указывает на директорию — выбирает самый новый `app-release-vX.Y.Z.apk` (по semver),
///   иначе любой `.apk` как fallback.
pub async fn find_latest_apk(path_or_dir: &str) -> Option<ApkCandidate> {
    let p = Path::new(path_or_dir);

    if p.is_file() {
        let filename = p.file_name()?.to_str()?.to_string();
        return Some(ApkCandidate {
            path: p.to_path_buf(),
            version: parse_version_from_filename(&filename),
            filename,
        });
    }

    if !p.is_dir() {
        return None;
    }

    let mut entries = fs::read_dir(p).await.ok()?;
    let mut candidates = Vec::new();

    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let filename = match path.file_name().and_then(|n| n.to_str()) {
            Some(f) => f.to_string(),
            None => continue,
        };
        if !filename.to_lowercase().ends_with(".apk") {
            continue;
        }

        candidates.push(ApkCandidate {
            version: parse_version_from_filename(&filename),
            filename,
            path,
        });
    }

    choose_best(candidates)
}
