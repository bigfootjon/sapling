/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This software may be used and distributed according to the terms of the
 * GNU General Public License version 2.
 */

use anyhow::anyhow;
use anyhow::Result;
use blobstore::Blobstore;
use context::CoreContext;
use filestore::FetchKey;
use filestore::FilestoreConfig;
use gix_hash::ObjectId;
use lazy_static::lazy_static;
use mononoke_types::hash;
use mononoke_types::hash::RichGitSha1;
use mononoke_types::hash::Sha256;
use mononoke_types::BasicFileChange;
use regex::Regex;

/// In line with https://github.com/git-lfs/git-lfs/blob/main/docs/spec.md
fn format_lfs_pointer(sha256: Sha256, size: u64) -> String {
    format!(
        "version https://git-lfs.github.com/spec/v1\noid sha256:{sha256}\nsize {size}\n",
        sha256 = sha256,
        size = size
    )
}

/// Given a file change generates a Git LFS pointer that points to acctual file contents
/// and stores it in the blobstore. Returns oid of the LFS pointer.
pub async fn generate_and_store_git_lfs_pointer<B: Blobstore + Clone + 'static>(
    blobstore: &B,
    filestore_config: FilestoreConfig,
    ctx: &CoreContext,
    basic_file_change: &BasicFileChange,
) -> Result<RichGitSha1> {
    let metadata = filestore::get_metadata(
        blobstore,
        ctx,
        &FetchKey::Canonical(basic_file_change.content_id()),
    )
    .await?
    .ok_or_else(|| anyhow!("Missing metadata for {}", basic_file_change.content_id()))?;
    let lfs_pointer = format_lfs_pointer(metadata.sha256, basic_file_change.size());
    let ((content_id, _size), fut) =
        filestore::store_bytes(blobstore, filestore_config, ctx, lfs_pointer.into());
    fut.await?;
    let oid = filestore::get_metadata(blobstore, ctx, &FetchKey::Canonical(content_id))
        .await?
        .ok_or_else(|| anyhow!("Missing metadata for {}", basic_file_change.content_id()))?
        .git_sha1;
    Ok(oid)
}

#[derive(Debug)]
pub struct LfsPointerData {
    pub version: String,
    pub sha256: hash::Sha256,
    pub size: u64,
    /// gitblob and gitid, where this metadata comes from. This is useful if we
    /// end up storing the metadata instead of the content (if the content cannot
    /// be found on the LFS server for example).
    pub gitblob: Vec<u8>,
    pub gitid: ObjectId,
    /// Whether the git lfs pointer in canonical format that would be generated
    /// if we were to generate it from scratch.
    pub is_canonical: bool,
}

lazy_static! {
    // Regex needs to match for the file to be attempted to be parsed as LFS.
    static ref LFS_MATCHER_RE: Regex = Regex::new(r"git-media|hawser|git-lfs").unwrap();
}

/// We will not try to parse any file bigger then this.
/// Any valid gitlfs metadata file should be smaller then this.
/// matches limit used by git-lfs:
/// https://github.com/git-lfs/git-lfs/blob/fc61febe9cc2d9ddc6ffe3e8d1ae546512632552/lfs/scanner.go#L12
const MAX_METADATA_LENGTH: usize = 1024;

const V1_ALIASES: [&str; 3] = [
    "http://git-media.io/v/2",            // alpha
    "https://hawser.github.com/spec/v1",  // pre-release
    "https://git-lfs.github.com/spec/v1", // public launch
];

/// Parses Git LFS pointer file into datastructure
/// see https://github.com/git-lfs/git-lfs/blob/main/docs/spec.md for format specification
///
/// Layout of the pointer:
/// | version https://git-lfs.github.com/spec/v1
/// | oid sha256:73e2200459562bb068f08e33210ed106014b877f878932b2147991e17a7c089b
/// | size 8423391
pub fn parse_lfs_pointer(gitblob: &[u8], gitid: ObjectId) -> Option<LfsPointerData> {
    if gitblob.len() > MAX_METADATA_LENGTH {
        return None;
    }

    let pointer = std::str::from_utf8(gitblob).ok()?;
    if !LFS_MATCHER_RE.is_match(pointer) {
        return None;
    }

    let (mut sha256, mut size, mut version) = (None, None, None);
    for line in pointer.lines() {
        let (k, v) = line.split_once(' ')?;
        match k {
            "oid" => {
                if sha256.is_some() {
                    return None;
                }
                sha256 = v.strip_prefix("sha256:")?.parse::<hash::Sha256>().ok();
            }
            "version" => {
                if version.is_some() {
                    return None;
                }
                if V1_ALIASES.contains(&v) {
                    version = Some(v.to_string());
                }
            }
            "size" => {
                if size.is_some() {
                    return None;
                }
                size = Some(v.parse::<u64>().ok()?);
            }
            _ => {
                // We're ignoring extra entries as Git LFS supports extensions to the format
                // and we don't want to know about those.
            }
        }
    }

    // only proceed if all fields are set
    let (version, sha256, size) = (version?, sha256?, size?);

    let is_canonical = format_lfs_pointer(sha256, size).as_bytes() == gitblob;

    Some(LfsPointerData {
        version,
        sha256,
        size,
        gitblob: gitblob.to_vec(),
        gitid,
        is_canonical,
    })
}
