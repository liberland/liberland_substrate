Tool that fetches all entries in storage (based on metadata) and tries to decode them. If it fails, it suggests that some migration is missing or doesn't work properly.

Usage:
```
yarn
npx ts-node src/index.ts
```

Example failure (indicates missing migration):
```
> npx ts-node src/index.ts
2024-05-22 17:12:48        API/INIT: RPC methods not decorated: chainSpec_v1_chainName, chainSpec_v1_genesisHash, chainSpec_v1_properties
Running checks...
2024-05-22 17:03:13        RPC-CORE: queryStorageAt(keys: Vec<StorageKey>, at?: BlockHash): Vec<StorageChangeSet>:: Unable to decode storage democracy.referendumInfoOf: entry 0:: createType(PalletDemocracyReferendumInfo):: {"_enum":{"Ongoing":"PalletDemocracyReferendumStatus","Finished":"{\"approved\":\"bool\",\"end\":\"u32\"}"}}:: Decoded input doesn't match input, received 0x009f505900026b04b6428f8c79e4868816ae606e69693dc73ab1ce5e53543753…409c000000000000000000000000000000c015da4c01c5000000000000000000 (112 bytes), created 0x009f505900026b04b6428f8c79e4868816ae606e69693dc73ab1ce5e53543753…00c015da4c01c500000000000000000000000000000000000000000000000000 (128 bytes)
Error: Unable to decode storage democracy.referendumInfoOf: entry 0:: createType(PalletDemocracyReferendumInfo):: {"_enum":{"Ongoing":"PalletDemocracyReferendumStatus","Finished":"{\"approved\":\"bool\",\"end\":\"u32\"}"}}:: Decoded input doesn't match input, received 0x009f505900026b04b6428f8c79e4868816ae606e69693dc73ab1ce5e53543753…409c000000000000000000000000000000c015da4c01c5000000000000000000 (112 bytes), created 0x009f505900026b04b6428f8c79e4868816ae606e69693dc73ab1ce5e53543753…00c015da4c01c500000000000000000000000000000000000000000000000000 (128 bytes)
Problem found, missing migration?
```

Example success (everything decoded correctly):
```
> npx ts-node src/index.ts
2024-05-22 17:12:48        API/INIT: RPC methods not decorated: chainSpec_v1_chainName, chainSpec_v1_genesisHash, chainSpec_v1_properties
Running checks...
All OK!
```
