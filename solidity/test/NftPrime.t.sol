// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "@openzeppelin/contracts/utils/Strings.sol";
import {Test, console} from "forge-std/Test.sol";
import {NftPrime} from "../src/NftPrime.sol";

contract NtfPrimeTest is Test {
    NftPrime public nftPrime;

    string constant SMALL_PRIMES = "smallPrimes";
    string constant LARGE_PRIMES = "largePrimes";
    string constant SMALL_COMPOSITES = "smallComposites";
    string constant LARGE_COMPOSITES = "largeComposites";

    function setUp() public {
        nftPrime = new NftPrime("", 14, 256);
    }

    function toBytes(uint256 number) internal pure returns(bytes memory) {
        return abi.encodePacked(number);
    }

    function getTestLength(string memory test) internal returns(uint256) {
        string[] memory runJsInputs = new string[](4);

        // node ./index.js {test} length
        runJsInputs[0]  = "node";
        runJsInputs[1]  = "./index.js";
        runJsInputs[2]  = test;
        runJsInputs[3]  = "length";

        bytes memory jsResult = vm.ffi(runJsInputs);
        (uint256 length) = abi.decode(jsResult, (uint256));
        return length;
    }

    function getTestByIndex(string memory test, uint256 index) internal returns(bytes memory,bytes memory,uint256) {
        string[] memory runJsInputs = new string[](4);

        // node ./index.js {test} {index}
        runJsInputs[0]  = "node";
        runJsInputs[1]  = "./index.js";
        runJsInputs[2]  = test;
        runJsInputs[3]  = Strings.toString(index);

        bytes memory jsResult = vm.ffi(runJsInputs);
        (bytes memory n, bytes memory d, uint256 s) = abi.decode(jsResult, (bytes, bytes, uint256));
        return (n, d, s);
    }

    function testInvalidPrimeParameters() public {
        vm.expectRevert("d must be greater than one");
        nftPrime.isPrime(toBytes(10), toBytes(0), 0);
        vm.expectRevert("d must be odd");
        nftPrime.isPrime(toBytes(10), toBytes(2), 4);
        vm.expectRevert("Invalid parameters d and s");
        nftPrime.isPrime(toBytes(10), toBytes(3), 4);
        assertTrue(nftPrime.isPrime(toBytes(13), toBytes(3), 2));
    }

    function testPrimalityForSmallPrimes() public {
        uint256 testLen = getTestLength(SMALL_PRIMES);
        for (uint256 i = 0; i < testLen; i++) {
            (bytes memory n, bytes memory d, uint256 s) = getTestByIndex(SMALL_PRIMES, i);
            bool isPrime = nftPrime.isPrime(n, d, s);
            assertTrue(isPrime);
        }
    }

    function testPrimalityForSmallComposites() public {
        uint256 testLen = getTestLength(SMALL_COMPOSITES);
        for (uint256 i = 0; i < testLen; i++) {
            (bytes memory n, bytes memory d, uint256 s) = getTestByIndex(SMALL_COMPOSITES, i);
            bool isPrime = nftPrime.isPrime(n, d, s);
            assertFalse(isPrime);
        }
    }

    function testPrimalityForLargePrimes() public {
        uint256 testLen = getTestLength(LARGE_PRIMES);
        for (uint256 i = 0; i < testLen; i++) {
            (bytes memory n, bytes memory d, uint256 s) = getTestByIndex(LARGE_PRIMES, i);
            bool isPrime = nftPrime.isPrime(n, d, s);
            assertTrue(isPrime);
        }
    }

    function testPrimalityForLargeComposites() public {
        uint256 testLen = getTestLength(LARGE_COMPOSITES);
        for (uint256 i = 0; i < testLen; i++) {
            (bytes memory n, bytes memory d, uint256 s) = getTestByIndex(LARGE_COMPOSITES, i);
            bool isPrime = nftPrime.isPrime(n, d, s);
            assertFalse(isPrime);
        }
    }

    function testZeroSParameter() public {
        vm.expectRevert("s must be greater than 0");
        nftPrime.mint(toBytes(3), toBytes(2), 0);
    }

    function testSizeForSmallNumbers() public {
        (bytes memory n, bytes memory d, uint256 s) = getTestByIndex(SMALL_PRIMES, 0);
        vm.expectRevert("Number not large enough");
        nftPrime.mint(n, d, s);
    }

    function testDuplicateMinting() public {
        uint256 testLen = getTestLength(LARGE_PRIMES);
        (bytes memory n, bytes memory d, uint256 s) = getTestByIndex(LARGE_PRIMES, testLen - 1);
        nftPrime.mint(n, d, s);
        vm.expectRevert("Prime was mined already");
        nftPrime.mint(n, d, s);
    }

    function testNotAPrimeMinting() public {
        uint256 testLen = getTestLength(LARGE_COMPOSITES);
        (bytes memory n, bytes memory d, uint256 s) = getTestByIndex(LARGE_COMPOSITES, testLen - 1);
        vm.expectRevert("Prime was mined already");
        nftPrime.mint(n, d, s);
    }

    function testSuccessfulMinting() public {
        bytes[] memory primes = new bytes[](4);
        for (uint256 i = 0; i < 4; i++) {
            uint256 testLen = getTestLength(LARGE_PRIMES);
            (bytes memory n, bytes memory d, uint256 s) = getTestByIndex(LARGE_PRIMES, testLen - 4 + i);
            nftPrime.mint(n, d, s);
            primes[i] = n;
        }
        assertEq(primes[0], nftPrime.getPrime(0).val);
        assertEq(primes[1], nftPrime.getPrime(1).val);
        assertEq(primes[2], nftPrime.getPrime(2).val);
        assertEq(primes[3], nftPrime.getPrime(3).val);
        assertEq(primes[0], nftPrime.getPrimes(0, 1)[0].val);
        assertEq(1, nftPrime.getPrimes(0, 1).length);
        assertEq(primes[1], nftPrime.getPrimes(0, 2)[1].val);
        assertEq(2, nftPrime.getPrimes(0, 2).length);
        assertEq(primes[2], nftPrime.getPrimes(1, 3)[1].val);
        assertEq(2, nftPrime.getPrimes(1, 3).length);
    }
}
