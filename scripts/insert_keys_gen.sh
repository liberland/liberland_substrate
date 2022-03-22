curl http://localhost:9933  -H "Content-Type:application/json;charset=utf-8" -d '{
    "jsonrpc":"2.0",
    "id":1,
    "method":"author_insertKey",
    "params": [
        "babe",
        "MNEMONIC_babe//babe",
        "ACCOUNT_babe"
    ]
}'

curl http://localhost:9933  -H "Content-Type:application/json;charset=utf-8" -d '{
    "jsonrpc":"2.0",
    "id":1,
    "method":"author_insertKey",
    "params": [
        "gran",
        "MNEMONIC_grandpa//grandpa",
        "ACCOUNT_grandpa"
    ]
}'

curl http://localhost:9933  -H "Content-Type:application/json;charset=utf-8" -d '{
    "jsonrpc":"2.0",
    "id":1,
    "method":"author_insertKey",
    "params": [
        "imol",
        "MNEMONIC_im_online//im_online",
        "ACCOUNT_im_online"
    ]
}'

curl http://localhost:9933  -H "Content-Type:application/json;charset=utf-8" -d '{
    "jsonrpc":"2.0",
    "id":1,
    "method":"author_insertKey",
    "params": [
        "authority_discovery",
        "MNEMONIC_authority_discovery//authority_discovery",
        "ACCOUNT_authority_discovery"
    ]
}'

