// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.17;

import {NftPrime} from "./NftPrime.sol";

// Contract used for simple deployment on local chain

contract NftPrimeLocal is NftPrime {
    constructor() NftPrime("https://localhost:8082/nft/prime", 14, 256, 50) {}
}