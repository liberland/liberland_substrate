create table if not exists networks (
    network text not null primary key,
    genesis text not null
);

create table if not exists syncstate (
    task_id text not null primary key,
    synced_block number not null default 0
);

create table if not exists eth_calls (
    id text not null primary key,
    signer_address text not null,
    request text not null,
    finished_tx text
);

create index finished_eth_calls on eth_calls(signer_address, finished_tx);

create table if not exists eth_transactions (
    eth_call_id text not null,
    hash text not null,
    max_fee_per_gas number not null,
    FOREIGN KEY(eth_call_id) REFERENCES eth_calls(id),
    PRIMARY KEY(eth_call_id, hash)
);

create table if not exists sub_calls (
    id text not null primary key,
    task_id text not null,
    bridge text not null check (bridge in ('LLM', 'LLD')),
    block_number number not null,
    amount blob not null,
    substrate_recipient blob not null,
    finished bool default false
);

create index finished_sub_calls on sub_calls(task_id);