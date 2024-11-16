/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This software may be used and distributed according to the terms of the
 * GNU General Public License version 2.
 */

use std::sync::Arc;

use futures::stream::BoxStream;
#[cfg(fbcode_build)]
pub use thrift_streaming::ChangeNotification;
#[cfg(fbcode_build)]
use thrift_streaming::ChangeNotificationResult;
#[cfg(fbcode_build)]
use thrift_streaming::EdenStartStatusUpdate;
#[cfg(fbcode_build)]
pub use thrift_streaming::LargeChangeNotification;
#[cfg(fbcode_build)]
pub use thrift_streaming::SmallChangeNotification;
#[cfg(fbcode_build)]
use thrift_streaming_clients::errors::StreamChangesSinceV2StreamError;
#[cfg(fbcode_build)]
use thrift_streaming_clients::errors::StreamStartStatusStreamError;
#[cfg(fbcode_build)]
use thrift_streaming_clients::StreamingEdenService;
use thrift_types::edenfs_clients::EdenService;

pub mod checkout;
pub mod fsutil;
pub mod instance;
mod mounttable;
pub mod redirect;
pub mod utils;

pub use instance::DaemonHealthy;
pub use instance::EdenFsInstance;

pub type EdenFsClient = Arc<dyn EdenService + Sync>;

#[cfg(fbcode_build)]
pub type StreamingEdenFsClient = Arc<dyn StreamingEdenService + Sync>;
#[cfg(fbcode_build)]
pub type StartStatusStream =
    BoxStream<'static, Result<EdenStartStatusUpdate, StreamStartStatusStreamError>>;
pub type ChangesSinceV2Stream =
    BoxStream<'static, Result<ChangeNotificationResult, StreamChangesSinceV2StreamError>>;
