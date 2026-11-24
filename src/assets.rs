//! Downloads the latest `nostalgia-assets` release and extracts it into
//! `~/.nostalgia/`.
//!
//! The latest-release API is checked on every invocation, including when a
//! cached archive is available, so this command requires an internet
//! connection. A cached archive is reused only when its SHA-256 digest matches
//! the latest release. Extraction updates files at matching archive paths but
//! leaves unrelated files and older versioned archives untouched.

use anyhow::{Context, ensure};
use reqwest::blocking::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tempfile::NamedTempFile;
use zip::ZipArchive;

/// GitHub API endpoint for the latest assets release.
const LATEST_RELEASE_URL: &str =
    "https://api.github.com/repos/nostalgia-rs/nostalgia-assets/releases/latest";
/// Required prefix for a release asset download URL.
const DOWNLOAD_URL_PREFIX: &str =
    "https://github.com/nostalgia-rs/nostalgia-assets/releases/download/";
/// Maximum accepted compressed asset archive size.
const MAX_DOWNLOAD_SIZE: u64 = 1024 * 1024 * 1024;
/// Maximum combined uncompressed size accepted from an asset archive.
const MAX_EXTRACTED_SIZE: u64 = 2 * 1024 * 1024 * 1024;

/// GitHub metadata for its latest release.
#[derive(Deserialize)]
struct Release {
    /// Files attached to the release.
    assets: Vec<ReleaseAsset>,
}

/// Metadata needed to download and verify a release asset.
#[derive(Deserialize)]
struct ReleaseAsset {
    /// Name used for the cached archive.
    name: String,
    /// GitHub's HTTPS URL for the release asset.
    browser_download_url: String,
    /// Published digest, expected in `sha256:<hex>` format.
    digest: Option<String>,
    /// Compressed archive size in bytes.
    size: u64,
}

/// Downloads and extracts the latest verified assets release into `~/.nostalgia/`.
///
/// A matching cached archive is reused, but the latest-release API is queried
/// on every call.
///
/// # Errors
///
/// Returns an error if the home directory is unavailable, the release cannot
/// be fetched or verified, or the archive cannot be extracted.
pub fn download_assets(home: Option<PathBuf>) -> anyhow::Result<()> {
    let home = home.context("Could not determine the home directory")?;
    let nostalgia_dir = home.join(".nostalgia");
    fs::create_dir_all(&nostalgia_dir)
        .with_context(|| format!("Could not create {}", nostalgia_dir.display()))?;

    let client = Client::builder()
        .user_agent(concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION")
        ))
        .timeout(Duration::from_secs(600))
        .build()
        .context("Could not create the GitHub download client")?;
    let release: Release = client
        .get(LATEST_RELEASE_URL)
        .send()
        .context("Could not fetch the latest assets release")?
        .error_for_status()
        .context("GitHub returned an error for the latest assets release")?
        .json()
        .context("Could not parse the latest assets release")?;
    let asset = release
        .assets
        .into_iter()
        .find(|asset| asset.name.to_ascii_lowercase().ends_with(".zip"))
        .context("The latest assets release does not contain a ZIP archive")?;

    ensure!(
        asset.browser_download_url.starts_with(DOWNLOAD_URL_PREFIX),
        "The assets release contains an unexpected download URL"
    );
    ensure!(
        asset.name.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
        }) && asset.name.to_ascii_lowercase().ends_with(".zip"),
        "The assets release contains an invalid archive filename"
    );
    ensure!(
        asset.size <= MAX_DOWNLOAD_SIZE,
        "The assets archive exceeds the 1 GiB download limit"
    );
    let expected_digest = asset
        .digest
        .as_deref()
        .and_then(|digest| digest.strip_prefix("sha256:"))
        .context("The assets release does not provide a SHA-256 digest")?;
    ensure!(
        expected_digest.len() == 64 && expected_digest.bytes().all(|byte| byte.is_ascii_hexdigit()),
        "The assets release provides an invalid SHA-256 digest"
    );

    let archive_path = nostalgia_dir.join(&asset.name);
    if archive_matches_digest(&archive_path, expected_digest)? {
        tracing::info!("Using the verified archive at {}", archive_path.display());
    } else {
        tracing::info!(
            "Downloading {} ({} MiB) to {}",
            asset.name,
            asset.size / (1024 * 1024),
            nostalgia_dir.display()
        );
        download_archive(
            &client,
            &asset,
            expected_digest,
            &nostalgia_dir,
            &archive_path,
        )?;
    }

    extract_archive(&archive_path, &nostalgia_dir)?;
    tracing::info!("Assets extracted to {}", nostalgia_dir.display());
    Ok(())
}

/// Checks whether a cached archive is a regular file with the expected SHA-256 digest.
///
/// Returns `false` when the archive does not exist or its digest does not match.
///
/// # Errors
///
/// Returns an error if the path cannot be inspected or the file cannot be read.
fn archive_matches_digest(path: &Path, expected_digest: &str) -> anyhow::Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_file() => {}
        Ok(_) => anyhow::bail!("{} is not a regular archive file", path.display()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => {
            return Err(error).with_context(|| format!("Could not inspect {}", path.display()));
        }
    }
    let file = File::open(path).with_context(|| format!("Could not read {}", path.display()))?;

    let mut reader = std::io::BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = vec![0; 64 * 1024];
    loop {
        let bytes_read = reader
            .read(&mut buffer)
            .with_context(|| format!("Could not read {}", path.display()))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()).eq_ignore_ascii_case(expected_digest))
}

/// Downloads an asset into a temporary file and persists it after verification.
///
/// # Errors
///
/// Returns an error if downloading, writing, verification, or persistence fails.
fn download_archive(
    client: &Client,
    asset: &ReleaseAsset,
    expected_digest: &str,
    destination: &Path,
    archive_path: &Path,
) -> anyhow::Result<()> {
    let response = client
        .get(&asset.browser_download_url)
        .send()
        .context("Could not download the assets archive")?
        .error_for_status()
        .context("GitHub returned an error while downloading the assets archive")?;
    let mut temporary_file =
        NamedTempFile::new_in(destination).context("Could not create a temporary archive file")?;
    stream_and_verify(response, &mut temporary_file, asset.size, expected_digest)?;
    temporary_file
        .as_file()
        .sync_all()
        .context("Could not flush the downloaded archive")?;

    match fs::symlink_metadata(archive_path) {
        Ok(metadata) if metadata.is_file() => {
            fs::remove_file(archive_path)
                .with_context(|| format!("Could not replace {}", archive_path.display()))?;
        }
        Ok(_) => anyhow::bail!("{} is not a regular archive file", archive_path.display()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(error)
                .with_context(|| format!("Could not inspect {}", archive_path.display()));
        }
    }
    temporary_file
        .persist(archive_path)
        .map_err(|error| error.error)
        .with_context(|| format!("Could not save {}", archive_path.display()))?;
    Ok(())
}

/// Streams a response into a temporary file while checking size and SHA-256.
///
/// # Errors
///
/// Returns an error if the stream cannot be read or written, or if its size or
/// digest differs from the release metadata.
fn stream_and_verify(
    mut response: impl Read,
    output: &mut NamedTempFile,
    expected_size: u64,
    expected_digest: &str,
) -> anyhow::Result<()> {
    let mut downloaded = 0u64;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0; 64 * 1024];

    loop {
        let bytes_read = response
            .read(&mut buffer)
            .context("Could not read the assets download")?;
        if bytes_read == 0 {
            break;
        }
        downloaded = downloaded
            .checked_add(bytes_read as u64)
            .context("The assets download size overflowed")?;
        ensure!(
            downloaded <= expected_size && downloaded <= MAX_DOWNLOAD_SIZE,
            "The assets download exceeded its expected size"
        );
        hasher.update(&buffer[..bytes_read]);
        output
            .write_all(&buffer[..bytes_read])
            .context("Could not write the downloaded assets archive")?;
    }

    ensure!(
        downloaded == expected_size,
        "The assets download size did not match the release metadata"
    );
    ensure!(
        format!("{:x}", hasher.finalize()).eq_ignore_ascii_case(expected_digest),
        "The assets archive SHA-256 digest did not match the release metadata"
    );
    Ok(())
}

/// Validates archive paths and expanded size, then extracts into the destination.
///
/// # Errors
///
/// Returns an error for an invalid ZIP, unsafe path, oversized archive, or
/// extraction failure.
fn extract_archive(archive_path: &Path, destination: &Path) -> anyhow::Result<()> {
    let file = File::open(archive_path)
        .with_context(|| format!("Could not open {}", archive_path.display()))?;
    let mut archive =
        ZipArchive::new(file).context("The downloaded assets file is not a valid ZIP archive")?;
    let mut extracted_size = 0u64;
    for index in 0..archive.len() {
        let entry = archive
            .by_index(index)
            .context("Could not read an entry in the assets archive")?;
        ensure!(
            entry.enclosed_name().is_some(),
            "The assets archive contains an unsafe path"
        );
        extracted_size = extracted_size
            .checked_add(entry.size())
            .context("The extracted assets size overflowed")?;
        ensure!(
            extracted_size <= MAX_EXTRACTED_SIZE,
            "The assets archive exceeds the 2 GiB extraction limit"
        );
    }

    archive
        .extract(destination)
        .with_context(|| format!("Could not extract assets into {}", destination.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::extract_archive;
    use sha2::{Digest, Sha256};
    use std::fs;
    use std::io::{Cursor, Write};
    use tempfile::NamedTempFile;
    use zip::ZipWriter;
    use zip::write::SimpleFileOptions;

    fn write_archive(path: &std::path::Path, entries: &[(&str, &[u8])]) {
        let mut archive = ZipWriter::new(Cursor::new(Vec::new()));
        for (name, contents) in entries {
            archive
                .start_file(name, SimpleFileOptions::default())
                .unwrap();
            archive.write_all(contents).unwrap();
        }
        fs::write(path, archive.finish().unwrap().into_inner()).unwrap();
    }

    #[test]
    fn extracts_and_updates_asset_archive_contents_in_the_target_directory() {
        let temporary_directory = tempfile::tempdir().unwrap();
        let archive_path = temporary_directory.path().join("assets.zip");
        let destination = temporary_directory.path().join(".nostalgia");
        write_archive(&archive_path, &[("pokemon/database.csv", b"category")]);

        extract_archive(&archive_path, &destination).unwrap();
        write_archive(&archive_path, &[("pokemon/database.csv", b"updated")]);
        extract_archive(&archive_path, &destination).unwrap();

        assert_eq!(
            fs::read(destination.join("pokemon/database.csv")).unwrap(),
            b"updated"
        );
    }

    #[test]
    fn rejects_archive_paths_that_escape_the_target_directory() {
        let temporary_directory = tempfile::tempdir().unwrap();
        let archive_path = temporary_directory.path().join("assets.zip");
        let destination = temporary_directory.path().join(".nostalgia");
        write_archive(&archive_path, &[("../outside.txt", b"unsafe")]);

        assert!(extract_archive(&archive_path, &destination).is_err());
        assert!(!temporary_directory.path().join("outside.txt").exists());
    }

    #[test]
    fn verifies_download_size_and_digest_while_streaming() {
        let contents = b"asset archive";
        let digest = format!("{:x}", Sha256::digest(contents));
        let expected_size = u64::try_from(contents.len()).unwrap();
        let mut output = NamedTempFile::new().unwrap();

        super::stream_and_verify(Cursor::new(contents), &mut output, expected_size, &digest)
            .unwrap();

        assert_eq!(fs::read(output.path()).unwrap(), contents);
    }

    #[test]
    fn rejects_downloads_with_an_invalid_digest() {
        let contents = b"asset archive";
        let expected_size = u64::try_from(contents.len()).unwrap();
        let mut output = NamedTempFile::new().unwrap();

        assert!(
            super::stream_and_verify(
                Cursor::new(contents),
                &mut output,
                expected_size,
                &"0".repeat(64),
            )
            .is_err()
        );
    }
}
