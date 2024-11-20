// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {ERC721} from "@openzeppelin/contracts/token/ERC721/ERC721.sol";
import {ERC721Enumerable} from "@openzeppelin/contracts/token/ERC721/extensions/ERC721Enumerable.sol";

contract NftPrime is ERC721Enumerable {
    string private _uri;

    constructor(string memory uri) ERC721("NFT Prime", "NFTP") {
        _uri = uri;
    }

    function _baseURI() internal view override returns (string memory) {
        return _uri;
    }
}