// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import {NftPrime} from "../src/NftPrime.sol";

contract NtfPrimeTest is Test {
    NftPrime public nftPrime;

    function setUp() public {
        nftPrime = new NftPrime("", 14, 256);
    }

    function toBytes(uint256 number) internal pure returns(bytes memory) {
        return abi.encodePacked(number);
    }

    function testInvalidPrimeParameters() public {
        vm.expectRevert("s and d must be greate than one");
        nftPrime.isPrime(toBytes(10), toBytes(0), 0);
        vm.expectRevert("d must be odd");
        nftPrime.isPrime(toBytes(10), toBytes(2), 4);
        vm.expectRevert("Invalid parameters d and s");
        nftPrime.isPrime(toBytes(10), toBytes(3), 4);
    }

    function testPrimalityForSmallPrimes() public {

    }

    function testPrimalityForSmallComposites() public {

    }

    function testPrimalityForLargePrimes() public {

    }

    function testPrimalityForLargeComposites() public {

    }

    function testSizeForSmallNumbers() public {

    }

    function testSizeForLargeNumbers() public {

    }

    function testDuplicateMinting() public {

    }

    function testNotAPrimeMinting() public {

    }

    function testSuccessfulMinting() public {

    }

    function testGetPrime() public {

    }

    function testPrimeListing() public {
        
    }
}
