use ethers::{
	abi::{AbiDecode, AbiEncode},
	types::Eip1559TransactionRequest,
	utils::to_checksum,
};
use futures::stream::StreamExt;
use primitive_types::H160;
use sqlx::{Acquire, SqliteConnection, SqlitePool};
use std::{str::FromStr, sync::Arc};
use subxt::utils::AccountId32;

use crate::{types::*, Result};
use private::*;

mod private {
	use super::*;
	use sqlx::Type;

	#[derive(Debug, Type)]
	#[sqlx(transparent)]
	pub struct DBH256(pub H256);

	#[derive(Debug, Type)]
	#[sqlx(transparent)]
	pub struct DBTaskId(pub String);

	impl From<&RelayId> for DBTaskId {
		fn from(r: &RelayId) -> Self {
			DBTaskId(r.clone().into())
		}
	}
	impl From<&WatcherId> for DBTaskId {
		fn from(r: &WatcherId) -> Self {
			DBTaskId(r.clone().into())
		}
	}
	impl From<&TaskId> for DBTaskId {
		fn from(r: &TaskId) -> Self {
			DBTaskId(r.clone().into())
		}
	}

	#[derive(Debug, Type)]
	#[sqlx(transparent)]
	pub struct DBCallId(pub String);

	impl From<&ReceiptId> for DBCallId {
		fn from(r: &ReceiptId) -> Self {
			Self(r.encode_hex())
		}
	}

	impl From<ReceiptId> for DBCallId {
		fn from(r: ReceiptId) -> Self {
			Self(r.encode_hex())
		}
	}

	impl From<&CallId> for DBCallId {
		fn from(r: &CallId) -> Self {
			Self(r.encode_hex())
		}
	}

	impl From<CallId> for DBCallId {
		fn from(r: CallId) -> Self {
			Self(r.encode_hex())
		}
	}

	impl From<DBCallId> for CallId {
		fn from(r: DBCallId) -> Self {
			H256::decode_hex(r.0).expect("DB corruption").into()
		}
	}

	#[derive(Debug, Type)]
	#[sqlx(transparent)]
	pub struct DBBridgeId(pub String);

	impl From<BridgeId> for DBBridgeId {
		fn from(r: BridgeId) -> Self {
			let s: &str = r.into();
			Self(s.to_string())
		}
	}

	impl From<DBBridgeId> for BridgeId {
		fn from(r: DBBridgeId) -> Self {
			r.0.as_str().try_into().expect("DB corruption")
		}
	}

	#[derive(Debug, Type)]
	#[sqlx(transparent)]
	pub struct DBAmount(pub String);

	impl From<Amount> for DBAmount {
		fn from(r: Amount) -> Self {
			Self(r.encode_hex())
		}
	}

	impl From<DBAmount> for Amount {
		fn from(r: DBAmount) -> Self {
			U256::decode_hex(r.0).expect("DB corruption")
		}
	}

	#[derive(Debug, Type)]
	#[sqlx(transparent)]
	pub struct DBAccountId32(pub String);

	impl From<AccountId32> for DBAccountId32 {
		fn from(r: AccountId32) -> Self {
			Self(format!("{r}"))
		}
	}

	impl From<DBAccountId32> for AccountId32 {
		fn from(r: DBAccountId32) -> Self {
			Self::from_str(r.0.as_str()).expect("DB corruption")
		}
	}

	#[derive(Debug, Type)]
	#[sqlx(transparent)]
	pub struct DBEthTxId(pub String);

	impl From<&H256> for DBEthTxId {
		fn from(r: &H256) -> Self {
			Self(r.encode_hex())
		}
	}

	impl From<H256> for DBEthTxId {
		fn from(r: H256) -> Self {
			Self(r.encode_hex())
		}
	}

	impl From<DBEthTxId> for H256 {
		fn from(r: DBEthTxId) -> Self {
			Self::decode_hex(r.0).expect("DB corruption")
		}
	}

	#[derive(Debug, Type)]
	#[sqlx(transparent)]
	pub struct DBEthAddress(pub String);

	impl From<&H160> for DBEthAddress {
		fn from(r: &H160) -> Self {
			Self(to_checksum(r, None))
		}
	}

	impl From<DBEthAddress> for H160 {
		fn from(r: DBEthAddress) -> Self {
			r.0.parse().expect("DB corruption")
		}
	}
}

/* GENERIC */
pub async fn get_synced_block<T: Into<DBTaskId>>(
	conn: &mut SqliteConnection,
	task_id: T,
) -> Result<BlockNumber> {
	let task_id = task_id.into();
	let synced_block = sqlx::query!(
		r#"select synced_block as "synced_block!: i64" from syncstate where task_id = ?1"#,
		task_id
	)
	.fetch_optional(&mut *conn)
	.await?;
	let synced_block = synced_block.map(|r| r.synced_block).unwrap_or(0);
	let synced_block = synced_block.try_into()?;
	Ok(synced_block)
}

pub async fn set_synced_block<T: Into<DBTaskId>>(
	conn: &mut SqliteConnection,
	task_id: T,
	number: BlockNumber,
) -> Result<()> {
	let number: i64 = number.try_into()?;
	let task_id = task_id.into();
	sqlx::query!(
				"insert into syncstate (task_id, synced_block) values (?1, ?2) on conflict(task_id) do update set synced_block = excluded.synced_block",
				task_id,
				number
			)
			.execute(&mut *conn)
			.await?;
	Ok(())
}

/* Substrate */
pub struct GetSubCallsRecord {
	pub call_id: CallId,
	pub bridge_id: BridgeId,
	pub eth_block_number: BlockNumber,
	pub amount: Amount,
	pub substrate_recipient: AccountId32,
}

pub async fn get_unfinished_sub_calls<T: Into<DBTaskId>>(
	conn: &mut SqliteConnection,
	task_id: T,
) -> Result<Vec<GetSubCallsRecord>> {
	let task_id = task_id.into();
	let res = sqlx::query!(
		r#"
			select
				id as "call_id: DBCallId",
				bridge as "bridge_id: DBBridgeId",
				block_number as "block_number!: i64",
				amount as "amount: DBAmount",
				substrate_recipient as "substrate_recipient: DBAccountId32"
			from sub_calls
			where task_id = ?1 and finished = false
		"#,
		task_id,
	)
	.map(|r| GetSubCallsRecord {
		call_id: r.call_id.into(),
		bridge_id: r.bridge_id.into(),
		eth_block_number: r.block_number.try_into().expect("DB corruption"),
		amount: r.amount.into(),
		substrate_recipient: r.substrate_recipient.into(),
	})
	.fetch_all(&mut *conn)
	.await?;
	Ok(res)
}

pub async fn insert_sub_call<R1, R2, B, A1, A2>(
	conn: &mut SqliteConnection,
	call_id: R1,
	task_id: R2,
	bridge_id: B,
	eth_block_number: BlockNumber,
	amount: A1,
	substrate_recipient: A2,
) -> Result<()>
where
	R1: Into<DBCallId>,
	R2: Into<DBTaskId>,
	B: Into<DBBridgeId>,
	A1: Into<DBAmount>,
	A2: Into<DBAccountId32>,
{
	let call_id = call_id.into();
	let task_id = task_id.into();
	let bridge_id = bridge_id.into();
	let amount = amount.into();
	let substrate_recipient = substrate_recipient.into();
	let eth_block_number: i64 = eth_block_number.try_into().expect("Too big block number");
	sqlx::query!(
		"
		insert into sub_calls
			(id, task_id, bridge, block_number, amount, substrate_recipient)
		values
			(?1, ?2, ?3, ?4, ?5, ?6)
		on conflict do nothing
		",
		call_id,
		task_id,
		bridge_id,
		eth_block_number,
		amount,
		substrate_recipient,
	)
	.execute(&mut *conn)
	.await?;
	Ok(())
}

pub async fn finish_sub_call<T: Into<DBCallId>>(
	conn: &mut SqliteConnection,
	call_id: T,
) -> Result<()> {
	let call_id = call_id.into();
	sqlx::query!("update sub_calls set finished = true where id = ?1", call_id)
		.execute(&mut *conn)
		.await?;
	Ok(())
}

/* Ethereum */

pub struct GetEthCallsRecord {
	pub call_id: CallId,
	pub request: Eip1559TransactionRequest,
	pub pendings: Vec<EthTrackedTx>,
}

pub async fn get_unfinished_eth_calls<T: Into<DBEthAddress>>(
	pool: &Arc<SqlitePool>,
	signer: T,
) -> Result<Vec<GetEthCallsRecord>> {
	let signer = signer.into();

	let mut conn = pool.acquire().await?;
	let mut conn2 = pool.acquire().await?;
	let mut call_stream = sqlx::query!(
		r#"select id as "call_id: DBCallId", request from eth_calls where signer_address = ?1 and finished_tx is null"#,
		signer,
	)
	.fetch(&mut conn);

	let mut calls = Vec::new();
	while let Some(row) = call_stream.next().await {
		let row = row?;
		let transactions = sqlx::query!(
			r#"select hash as "hash: DBEthTxId", max_fee_per_gas as "max_fee_per_gas!: i64" from eth_transactions where eth_call_id = ?1"#,
			row.call_id
		)
		.fetch_all(&mut conn2)
		.await?;
		let pendings: Result<Vec<EthTrackedTx>> = transactions
			.into_iter()
			.map(|tx_row| {
				Ok::<EthTrackedTx, eyre::Report>(EthTrackedTx {
					hash: tx_row.hash.into(),
					max_fee_per_gas: tx_row.max_fee_per_gas.into(),
				})
			})
			.collect();
		let pendings = pendings?;
		calls.push(GetEthCallsRecord {
			call_id: row.call_id.into(),
			request: serde_json::from_str(&row.request)?,
			pendings,
		});
	}
	Ok(calls)
}

pub async fn insert_eth_call<R1, R2>(
	conn: &mut SqliteConnection,
	call_id: R1,
	signer: R2,
	request: &Eip1559TransactionRequest,
) -> Result<()>
where
	R1: Into<DBCallId>,
	R2: Into<DBEthAddress>,
{
	let signer = signer.into();
	let call_id = call_id.into();
	let encoded_request = serde_json::to_string(request)?;

	sqlx::query!(
		"insert into eth_calls (id, signer_address, request) values (?1, ?2, ?3);",
		call_id,
		signer,
		encoded_request,
	)
	.execute(conn)
	.await?;
	Ok(())
}

pub async fn insert_eth_tx<R, T, A>(
	conn: &mut SqliteConnection,
	call_id: R,
	hash: T,
	max_fee_per_gas: A,
) -> Result<()>
where
	R: Into<DBCallId>,
	T: Into<DBEthTxId>,
	A: Into<DBAmount>,
{
	let call_id = call_id.into();
	let hash = hash.into();
	let max_fee_per_gas = max_fee_per_gas.into();

	sqlx::query!(
		"insert into eth_transactions (eth_call_id, hash, max_fee_per_gas) values (?1, ?2, ?3);",
		call_id,
		hash,
		max_fee_per_gas,
	)
	.execute(conn)
	.await?;
	Ok(())
}

pub async fn drop_eth_tx<T>(conn: &mut SqliteConnection, hash: T) -> Result<()>
where
	T: Into<DBEthTxId>,
{
	let hash = hash.into();

	sqlx::query!("delete from eth_transactions where hash = ?1", hash)
		.execute(conn)
		.await?;
	Ok(())
}

pub async fn set_eth_call_request<T: Into<DBCallId>>(
	conn: &mut SqliteConnection,
	call_id: T,
	request: &Eip1559TransactionRequest,
) -> Result<()> {
	let call_id = call_id.into();
	let request = serde_json::to_string(request)?;
	sqlx::query!("update eth_calls set request = ?1 where id = ?2;", request, call_id)
		.execute(conn)
		.await?;
	Ok(())
}

pub async fn finish_eth_call<R, H>(pool: &Arc<SqlitePool>, call_id: R, hash: H) -> Result<()>
where
	R: Into<DBCallId>,
	H: Into<DBEthTxId>,
{
	let mut conn = pool.acquire().await?;
	let mut db_tx = conn.begin().await?;

	let call_id = call_id.into();
	let hash = hash.into();
	sqlx::query!("update eth_calls set finished_tx = ?2 where id = ?1", call_id, hash)
		.execute(&mut db_tx)
		.await?;

	sqlx::query!("delete from eth_transactions where eth_call_id = ?1", call_id)
		.execute(&mut db_tx)
		.await?;
	db_tx.commit().await?;

	Ok(())
}
