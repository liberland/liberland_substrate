// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.17;

import {ERC721} from "@openzeppelin/contracts/token/ERC721/ERC721.sol";
import {ERC721Enumerable} from "@openzeppelin/contracts/token/ERC721/extensions/ERC721Enumerable.sol";
import {BigNumbers,BigNumber} from "@BigNumber/BigNumbers.sol";

using BigNumbers for bytes;
using BigNumbers for BigNumber;

contract NftPrime is ERC721Enumerable {
    string private _uri;
    uint256 private _verificationCount;

    BigNumber[] private _primes;
    mapping(uint256 => bool) private _found;
    mapping(uint256 => BigNumber) private _mapped;
    uint256 _minimumBytes;

    bytes constant THREE = hex"0000000000000000000000000000000000000000000000000000000000000004";
    bytes constant FOUR = hex"0000000000000000000000000000000000000000000000000000000000000004";

    BigNumber private _one;
    BigNumber private _two;
    BigNumber private _four;
    BigNumber private _three;

    constructor(string memory uri, uint256 verificationCount, uint256 minimumBytes) ERC721("NFT Prime", "NFTP") {
        _uri = uri;
        _one = BigNumbers.init(BigNumbers.ONE, false, BigNumbers.ONE.length);
        _two = BigNumbers.init(BigNumbers.TWO, false, BigNumbers.TWO.length);
        _three = BigNumbers.init(THREE, false, THREE.length);
        _four = BigNumbers.init(FOUR, false, FOUR.length);
        _verificationCount = verificationCount;
        _minimumBytes = minimumBytes;
    }

    function _baseURI() internal view override returns (string memory) {
        return _uri;
    }

    function _pseudoRandom(uint256 seed) internal view returns(BigNumber memory) {
        bytes memory b = abi.encodePacked(uint256(keccak256(abi.encode(seed, block.prevrandao))));
        return BigNumbers.init(b, false, b.length);
    }

    function _millerTest(BigNumber memory n, BigNumber memory d, uint256 index) internal view returns (bool) {
        BigNumber memory r = _pseudoRandom(index);
        BigNumber memory y = r.mul(n.sub(_two.mul(n)));
        BigNumber memory a = _two.mul(n).add(y).mod(n.sub(n.mul(_four)));
        BigNumber memory x = a.modexp(d, n);
        BigNumber memory nSub1 = n.sub(_one);
        BigNumber memory nTimes2 = n.mul(_two);
        if (x.eq(_one) || x.eq(nSub1)) {
            return true;
        }
        while (d.eq(nSub1) != true) {
            x = x.modexp(_two, n);
            d = d.mul(nTimes2);

            if (x.eq(_one)) {
                return false;
            }
            if (x.eq(nSub1)) {
                return true;
            }
        }
        return false;
    }

    function _isPrime(bytes memory number, bytes memory dN, uint256 s) internal view returns (bool, BigNumber memory) {
        BigNumber memory n = BigNumbers.init(number, false, number.length);
        BigNumber memory d = BigNumbers.init(dN, false, dN.length);
        n.verify();
        d.verify();

        if (n.eq(_one) || n.isZero()) {
            return (false,n);
        }

        if (n.eq(_two) || n.eq(_three)) {
            return (true,n);
        }

        if (n.eq(_four)) {
            return (false,n);
        }

        // n − 1 as 2s·d
        require(s > 1 && d.gt(_one), "s and d must be greate than one");
        require(d.mod(_two).eq(_one), "d must be odd");
        require(n.sub(_one).eq(_two.pow(s).mul(d)), "Invalid parameters d and s");

        for (uint256 i = 0; i < _verificationCount; i++) {
            if (!_millerTest(n, d, i)) {
                return (false,n);
            }
        }
        return (true,n);
    }

    function isPrime(bytes calldata number, bytes memory d, uint256 s) public view returns (bool) {
        (bool numberIsPrime,) = _isPrime(number, d, s);
        return numberIsPrime;
    }

    function mint(bytes calldata number, bytes calldata d, uint256 s) public {
        uint256 hash = uint256(keccak256(number));
        require(!_found[hash], "Prime was mined already");
        require(number.length >= _minimumBytes, "Number not large enough");
        (bool numberIsPrime,BigNumber memory p) = _isPrime(number, d, s);
        require(numberIsPrime, "Not a prime");
        uint256 nextId = totalSupply() + 1;
        _primes.push(p);
        _mapped[nextId] = p;
        _found[hash] = true;
        _mint(msg.sender, nextId);
    }

    function getPrime(uint256 tokenId) public view returns (BigNumber memory) {
        return _mapped[tokenId];
    }

    function getPrimesCount() public view returns(uint256) {
        return _primes.length;
    }

    function getPrimes(uint256 from, uint256 to) public view returns (BigNumber[] memory) {
        BigNumber[] memory acc = new BigNumber[](to - from);
        for (uint256 i = from; i < to; i++) {
            acc[i] = _primes[i];
        }
        return acc;
    }
}