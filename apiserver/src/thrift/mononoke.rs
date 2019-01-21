// Copyright (c) 2018-present, Facebook, Inc.
// All Rights Reserved.
//
// This software may be used and distributed according to the terms of the
// GNU General Public License version 2 or any later version.

use std::convert::TryInto;
use std::sync::Arc;

use apiserver_thrift::server::MononokeApiservice;
use apiserver_thrift::services::mononoke_apiservice::GetRawExn;
use apiserver_thrift::types::MononokeGetRawParams;
use errors::ErrorKind;
use failure::err_msg;
use futures::{Future, IntoFuture};
use futures_ext::{BoxFuture, FutureExt};
use futures_stats::Timed;
use scuba_ext::{ScubaSampleBuilder, ScubaSampleBuilderExt};
use slog::Logger;
use time_ext::DurationExt;

use super::super::actor::{Mononoke, MononokeRepoResponse};

#[derive(Clone)]
pub struct MononokeAPIServiceImpl {
    addr: Arc<Mononoke>,
    logger: Logger,
    scuba_builder: ScubaSampleBuilder,
}

impl MononokeAPIServiceImpl {
    pub fn new(addr: Arc<Mononoke>, logger: Logger, scuba_table_name: Option<String>) -> Self {
        let mut scuba_builder = if let Some(table_name) = scuba_table_name {
            ScubaSampleBuilder::new(table_name)
        } else {
            ScubaSampleBuilder::with_discard()
        };

        scuba_builder.add_common_server_data();

        Self {
            addr,
            logger,
            scuba_builder,
        }
    }
}

impl MononokeApiservice for MononokeAPIServiceImpl {
    fn get_raw(&self, params: MononokeGetRawParams) -> BoxFuture<Vec<u8>, GetRawExn> {
        let mut scuba = self.scuba_builder.clone();

        scuba
            .add_common_server_data()
            .add("type", "thrift")
            .add("method", "get_raw")
            .add(
                "params",
                serde_json::to_string(&params)
                    .unwrap_or_else(|_| "Error converting request to json".to_string()),
            )
            .add(
                "path",
                String::from_utf8(params.path.clone())
                    .unwrap_or("Invalid UTF-8 in path".to_string()),
            )
            .add("changeset", params.changeset.clone());

        params
            .try_into()
            .into_future()
            .from_err()
            .and_then({
                cloned!(self.addr);
                move |param| addr.send_query(param)
            })
            .and_then(|resp: MononokeRepoResponse| match resp {
                MononokeRepoResponse::GetRawFile { content } => Ok(content.to_vec()),
                _ => Err(ErrorKind::InternalError(err_msg(
                    "Actor returned wrong response type to query".to_string(),
                ))),
            })
            .map_err(move |e| GetRawExn::e(e.into()))
            .boxify()
            .timed({
                move |stats, resp| {
                    scuba
                        .add_future_stats(&stats)
                        .add("response_time", stats.completion_time.as_micros_unchecked())
                        .add("response_size", resp.map(|vec| vec.len()).unwrap_or(0));

                    scuba.log();

                    Ok(())
                }
            })
    }
}
