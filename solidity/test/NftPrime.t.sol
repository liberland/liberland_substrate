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

    struct TestData {
        bytes n;
        bytes d;
        uint256 s;
    }

    function setUp() public {
        nftPrime = new NftPrime("", 14, 256);
    }

    function toBytes(uint256 number) internal pure returns(bytes memory) {
        return abi.encode(number);
    }

    function getTest(string memory test) internal returns(TestData[] memory) {
        string[] memory runJsInputs = new string[](4);

        // node ./index.js {test} {index}
        runJsInputs[0]  = "node";
        runJsInputs[1]  = "./test/index.js";
        runJsInputs[2]  = test;

        bytes memory jsResult = vm.ffi(runJsInputs);
        (TestData[] memory result) = abi.decode(jsResult, (TestData[]));
        return result;
    }

    function testInvalidPrimeParameters() public {
        vm.expectRevert("d must be greater or equal to one");
        nftPrime.isPrime(toBytes(10), toBytes(0), 0);
        vm.expectRevert("d must be odd");
        nftPrime.isPrime(toBytes(10), toBytes(2), 4);
        vm.expectRevert("Invalid parameters d and s");
        nftPrime.isPrime(toBytes(10), toBytes(3), 4);
        assertTrue(nftPrime.isPrime(toBytes(13), toBytes(3), 2));
    }

    function testPrimalityForSmallPrimes() public {
        TestData[] memory _smallPrimeTests = getTest(SMALL_PRIMES);
        for (uint256 i = 0; i < _smallPrimeTests.length; i++) {
            TestData memory test = _smallPrimeTests[i];
            bool isPrime = nftPrime.isPrime(test.n, test.d, test.s);
            assertTrue(isPrime);
        }
    }

    function testPrimalityForSmallComposites() public {
        TestData[] memory _smallCompositeTests = getTest(SMALL_COMPOSITES);
        for (uint256 i = 0; i < _smallCompositeTests.length; i++) {
            TestData memory test = _smallCompositeTests[i];
            bool isPrime = nftPrime.isPrime(test.n, test.d, test.s);
            assertFalse(isPrime);
        }
    }

    /*function testPrimalityForLargePrimes() public {
        TestData[] memory _largePrimeTests = getTest(LARGE_PRIMES);
        for (uint256 i = 0; i < _largePrimeTests.length; i++) {
            TestData memory test = _largePrimeTests[i];
            bool isPrime = nftPrime.isPrime(test.n, test.d, test.s);
            assertTrue(isPrime);
        }
    }

    function testPrimalityForLargeComposites() public {
        TestData[] memory _largeCompositeTests = getTest(LARGE_COMPOSITES);
        for (uint256 i = 0; i < _largeCompositeTests.length; i++) {
            TestData memory test = _largeCompositeTests[i];
            bool isPrime = nftPrime.isPrime(test.n, test.d, test.s);
            assertFalse(isPrime);
        }
    }

    function testZeroSParameter() public {
        vm.expectRevert("s must be greater than 0");
        nftPrime.mint(toBytes(3), toBytes(2), 0);
    }

    function testSizeForSmallNumbers() public {
        TestData[] memory _smallPrimeTests = getTest(SMALL_PRIMES);
        TestData memory test = _smallPrimeTests[0];
        vm.expectRevert("Number not large enough");
        nftPrime.mint(test.n, test.d, test.s);
    }

    function testDuplicateMinting() public {
        TestData[] memory _largePrimeTests = getTest(LARGE_PRIMES);
        TestData memory test = _largePrimeTests[_largePrimeTests.length - 1];
        nftPrime.mint(test.n, test.d, test.s);
        vm.expectRevert("Prime was mined already");
        nftPrime.mint(test.n, test.d, test.s);
    }

    function testNotAPrimeMinting() public {
        TestData[] memory _largeCompositeTests = getTest(LARGE_COMPOSITES);
        TestData memory test = _largeCompositeTests[_largeCompositeTests.length - 1];
        vm.expectRevert("Not a prime");
        nftPrime.mint(test.n, test.d, test.s);
    }

    function testSuccessfulMinting() public {
        TestData[] memory _largePrimeTests = getTest(LARGE_PRIMES);
        bytes[] memory primes = new bytes[](4);
        for (uint256 i = 0; i < 4; i++) {
            TestData memory test = _largePrimeTests[_largePrimeTests.length - 4 + i];
            nftPrime.mint(test.n, test.d, test.s);
            primes[i] = test.n;
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
    }*/
}
