// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.17;

import {ERC721} from "@openzeppelin/contracts/token/ERC721/ERC721.sol";
import {ERC721Enumerable} from "@openzeppelin/contracts/token/ERC721/extensions/ERC721Enumerable.sol";
import {BigNumbers,BigNumber} from "@BigNumber/BigNumbers.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";

contract NftPrimeEvents {
    event SetFee(uint256 indexed fee);
    event Withdraw(uint256 indexed amount);
}

contract NftPrime is NftPrimeEvents,ERC721Enumerable,Ownable {
    using BigNumbers for bytes;
    using BigNumbers for BigNumber;

    string private _uri;
    uint256 private _verificationCount;
    uint256 _minimumBytes;
    uint256 _fee;
    uint256 _paid;
    BigNumber[] private _primes;
    mapping(uint256 => bool) private _found;
    mapping(uint256 => uint256) private _mapped;

    bytes constant ONE = hex"0000000000000000000000000000000000000000000000000000000000000001";
    bytes constant TWO = hex"0000000000000000000000000000000000000000000000000000000000000002";
    bytes constant THREE = hex"0000000000000000000000000000000000000000000000000000000000000003";
    bytes constant FOUR = hex"0000000000000000000000000000000000000000000000000000000000000004";

    BigNumber private _one;
    BigNumber private _two;
    BigNumber private _four;
    BigNumber private _three;

    constructor(string memory uri, uint256 verificationCount, uint256 minimumBytes, uint256 fee) ERC721("NFT Prime", "NFTP") Ownable(msg.sender) {
        _uri = uri;
        _one = ONE.init(false);
        _two = TWO.init(false);
        _three = THREE.init(false);
        _four = FOUR.init(false);
        _verificationCount = verificationCount;
        _minimumBytes = minimumBytes;
        _fee = fee;
    }

    function _baseURI() internal view override returns (string memory) {
        return _uri;
    }

    function _pseudoRandom(uint256 seed, BigNumber memory n) internal view returns(BigNumber memory) {
        bytes memory b = abi.encodePacked(uint256(keccak256(abi.encode(seed, block.prevrandao))));
        return b.init(false).mod(n);
    }

    function _millerTest(BigNumber memory n, BigNumber memory d, uint256 index) internal view returns (bool) {
        BigNumber memory r = _pseudoRandom(index, n);
        BigNumber memory y = r.mul(n.sub(_two)); // r * (n - 2)
        BigNumber memory a = _two.add(y.mod(n.sub(_four))); // 2 + (y % (n - 4n))
        BigNumber memory x = _safeModExp(a, d, n); // a^d % n
        BigNumber memory nSub1 = n.sub(_one);
        if (x.eq(_one) || x.eq(nSub1)) {
            return true;
        }
        while (d.eq(nSub1) != true) {
            x = _safeModExp(x, _two, n); // x = (x * x) % n;
            d = d.mul(_two); // d *= 2;

            if (x.eq(_one)) {
                return false;
            }
            if (x.eq(nSub1)) {
                return true;
            }
        }
        return false;
    }

    function _safeModExp(BigNumber memory n, BigNumber memory exponent, BigNumber memory m) internal view returns(BigNumber memory) {
        return exponent.isZero() ? _one : n.modexp(exponent, m);
    }

    function _safePow(BigNumber memory n, uint256 exponent) internal view returns (BigNumber memory) {
        return exponent == 0 ? _one : n.pow(exponent);
    }

    function _isPrime(bytes memory number, bytes memory dN, uint256 s) internal view returns (bool, BigNumber memory) {
        BigNumber memory n = number.init(false);
        BigNumber memory d = dN.init(false);
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

        require(d.gt(_one) || d.eq(_one), "d must be greater or equal to one");
        require(d.mod(_two).eq(_one), "d must be odd");
        require(n.sub(_one).eq(_safePow(_two, s).mul(d)), "Invalid parameters d and s");

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

    function mint(bytes calldata number, bytes calldata d, uint256 s) public payable {
        require(msg.value == _fee, "Minting fee must be paid");
        require(s > 0, "s must be greater than 0");
        uint256 hash = uint256(keccak256(number));
        require(!_found[hash], "Prime was mined already");
        require(number.length >= _minimumBytes, "Number not large enough");
        (bool numberIsPrime,BigNumber memory p) = _isPrime(number, d, s);
        require(numberIsPrime, "Not a prime");
        uint256 nextId = totalSupply();
        _primes.push(p);
        _mapped[nextId] = _primes.length - 1;
        _found[hash] = true;
        _mint(msg.sender, nextId);
        _paid += msg.value;
    }

    function getPrime(uint256 tokenId) public view returns (BigNumber memory) {
        return _primes[_mapped[tokenId]];
    }

    function getPrimesCount() public view returns(uint256) {
        return _primes.length;
    }

    function getPrimesOwnedBy(address owner, uint256 from, uint256 to) public view returns(BigNumber[] memory,uint256[] memory) {
        BigNumber[] memory numbers = new BigNumber[](to - from);
        uint256[] memory ids = new uint256[](to - from);
        for (uint256 i = 0; i < (to - from); i++) {
            uint256 id = tokenOfOwnerByIndex(owner, i + from);
            ids[i] = id;
            numbers[i] = _primes[_mapped[id]];
        }
        return (numbers,ids);
    }

    function getPrimes(uint256 from, uint256 to) public view returns (BigNumber[] memory) {
        BigNumber[] memory acc = new BigNumber[](to - from);
        for (uint256 i = 0; i < (to - from); i++) {
            acc[i] = _primes[i + from];
        }
        return acc;
    }

    function getFee() public view returns(uint256) {
        return _fee;
    }

    function setFee(uint256 fee) public onlyOwner() {
        _fee = fee;
        emit SetFee(fee);
    }

    function getPaid() public view returns(uint256) {
        return _paid;
    }

    function withdraw() public onlyOwner() {
        require(_paid > 0, "no funds found");
        (bool ok,) = msg.sender.call{ value: _paid }("");
        require(ok, "transaction failed");
        emit Withdraw(_paid);
        _paid = 0;
    }
}