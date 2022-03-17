curl http://localhost:9933  -H "Content-Type:application/json;charset=utf-8" -d '{
    "jsonrpc":"2.0",
    "id":1,
    "method":"author_insertKey",
    "params": [
        "babe",
        "<REPLACE WITH MNEMONIC>//babe",
        "<REPLACE WITH Account ID>"
    ]
}'

curl http://localhost:9933  -H "Content-Type:application/json;charset=utf-8" -d '{
    "jsonrpc":"2.0",
    "id":1,
    "method":"author_insertKey",
    "params": [
        "gran",
        "<REPLACE WITH MNEMONIC>//grandpa",
        "<REPLACE WITH Account ID>"
    ]
}'

curl http://localhost:9933  -H "Content-Type:application/json;charset=utf-8" -d '{
    "jsonrpc":"2.0",
    "id":1,
    "method":"author_insertKey",
    "params": [
        "imol",
        ""<REPLACE WITH MNEMONIC>//im_online",
        "<REPLACE WITH Account ID>"
    ]
}'

curl http://localhost:9933  -H "Content-Type:application/json;charset=utf-8" -d '{
    "jsonrpc":"2.0",
    "id":1,
    "method":"author_insertKey",
    "params": [
        "audi",
        "<REPLACE WITH MNEMONIC>//authority_discovery",
        "<REPLACE WITH Account ID>"
    ]
}'
