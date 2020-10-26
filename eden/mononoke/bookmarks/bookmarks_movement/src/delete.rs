/*
 * Copyright (c) Facebook, Inc. and its affiliates.
 *
 * This software may be used and distributed according to the terms of the
 * GNU General Public License version 2.
 */

use std::collections::HashMap;

use blobrepo::BlobRepo;
use bookmarks::{BookmarkUpdateReason, BundleReplay};
use bookmarks_types::BookmarkName;
use bytes::Bytes;
use context::CoreContext;
use metaconfig_types::{BookmarkAttrs, InfinitepushParams, SourceControlServiceParams};
use mononoke_types::ChangesetId;
use repo_read_write_status::RepoReadWriteFetcher;

use crate::repo_lock::check_repo_lock;
use crate::restrictions::{BookmarkKind, BookmarkKindRestrictions, BookmarkMoveAuthorization};
use crate::BookmarkMovementError;

pub struct DeleteBookmarkOp<'op> {
    bookmark: &'op BookmarkName,
    old_target: ChangesetId,
    reason: BookmarkUpdateReason,
    auth: BookmarkMoveAuthorization<'op>,
    kind_restrictions: BookmarkKindRestrictions,
    pushvars: Option<&'op HashMap<String, Bytes>>,
    bundle_replay: Option<&'op dyn BundleReplay>,
}

#[must_use = "DeleteBookmarkOp must be run to have an effect"]
impl<'op> DeleteBookmarkOp<'op> {
    pub fn new(
        bookmark: &'op BookmarkName,
        old_target: ChangesetId,
        reason: BookmarkUpdateReason,
    ) -> DeleteBookmarkOp<'op> {
        DeleteBookmarkOp {
            bookmark,
            old_target,
            reason,
            auth: BookmarkMoveAuthorization::User,
            kind_restrictions: BookmarkKindRestrictions::AnyKind,
            pushvars: None,
            bundle_replay: None,
        }
    }

    /// This bookmark change is for an authenticated named service.  The change
    /// will be checked against the service's write restrictions.
    pub fn for_service(
        mut self,
        service_name: impl Into<String>,
        params: &'op SourceControlServiceParams,
    ) -> Self {
        self.auth = BookmarkMoveAuthorization::Service(service_name.into(), params);
        self
    }

    pub fn only_if_scratch(mut self) -> Self {
        self.kind_restrictions = BookmarkKindRestrictions::OnlyScratch;
        self
    }

    pub fn only_if_public(mut self) -> Self {
        self.kind_restrictions = BookmarkKindRestrictions::OnlyPublic;
        self
    }

    pub fn with_pushvars(mut self, pushvars: Option<&'op HashMap<String, Bytes>>) -> Self {
        self.pushvars = pushvars;
        self
    }

    pub fn with_bundle_replay_data(mut self, bundle_replay: Option<&'op dyn BundleReplay>) -> Self {
        self.bundle_replay = bundle_replay;
        self
    }

    pub async fn run(
        self,
        ctx: &'op CoreContext,
        repo: &'op BlobRepo,
        infinitepush_params: &'op InfinitepushParams,
        bookmark_attrs: &'op BookmarkAttrs,
        repo_read_write_fetcher: &'op RepoReadWriteFetcher,
    ) -> Result<(), BookmarkMovementError> {
        let kind = self
            .kind_restrictions
            .check_kind(infinitepush_params, self.bookmark)?;

        self.auth
            .check_authorized(ctx, bookmark_attrs, self.bookmark, kind)?;

        if kind == BookmarkKind::Scratch || bookmark_attrs.is_fast_forward_only(self.bookmark) {
            // Cannot delete scratch or fast-forward-only bookmarks.
            return Err(BookmarkMovementError::DeletionProhibited {
                bookmark: self.bookmark.clone(),
            });
        }

        check_repo_lock(repo_read_write_fetcher, kind, self.pushvars).await?;

        let mut txn = repo.update_bookmark_transaction(ctx.clone());
        txn.delete(
            self.bookmark,
            self.old_target,
            self.reason,
            self.bundle_replay,
        )?;

        let ok = txn.commit().await?;
        if !ok {
            return Err(BookmarkMovementError::TransactionFailed);
        }

        Ok(())
    }
}
