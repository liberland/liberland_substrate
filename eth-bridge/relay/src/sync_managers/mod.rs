mod ethereum;
mod substrate;

use crate::{db, types::*, Result};
use async_stream::try_stream;
use futures::{Stream, StreamExt};
use sqlx::sqlite::SqlitePool;
use std::sync::Arc;

pub use ethereum::{Ethereum, EthereumSyncTarget};
pub use substrate::{Substrate, SubstrateSyncTarget};
