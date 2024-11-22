const { assert } = require("console");
const fs = require("fs");
/**
 * @type number[]
 */
const smallPrimes = JSON.parse(fs.openSync("./first-primes.json", "r"));
/**
 * @type string[]
 */
const largePrimes = JSON.parse(fs.openSync("./large-primes.json", "r"));

/**
 * @type number[]
 */
const smallComposites = JSON.parse(fs.openSync("./first-composites.json", "r"));
/**
 * @type string[]
 */
const largeComposites = JSON.parse(fs.openSync("./large-composites.json", "r"));

const results = {
    smallPrimes: [],
    largePrimes: [],
    smallComposites: [],
    largeComposites: [],
};

smallPrimes.forEach((value) => {
    const [numberIsPrime, d, s] = isPrime(BigInt(value));
    assert(numberIsPrime);
    results.smallPrimes.push({ n: value.toString(), d, s });
});

largePrimes.forEach((value) => {
    const [numberIsPrime, d, s] = isPrime(BigInt(value));
    assert(numberIsPrime);
    results.largePrimes.push({ n: value.toString(), d, s });
});

smallComposites.forEach((value) => {
    
});

// This function is called
// for all k trials. It returns
// false if n is composite and
// returns false if n is
// probably prime. d is an odd
// number such that d*2<sup>r</sup> = n-1
// for some r >= 1
function millerTest(d, n)
{
    // (JML- all literal integers converted to use n suffix denoting BigInt)
        
    // Pick a random number in [2..n-2]
    // Corner cases make sure that n > 4
    /* 
        JML- I can't mix the Number returned by Math.random with
        operations involving BigInt. The workaround is to create a random integer 
        with precision 6 and convert it to a BigInt.
    */  
    const r = BigInt(Math.floor(Math.random() * 100_000))
    // JML- now I have to divide by the multiplier used above (BigInt version)
    const y = r*(n-2n)/100_000n
    let a = 2n + y % (n - 4n);

    // Compute a^d % n
    let x = power(a, d, n);

    if (x == 1n || x == n-1n)
        return true;

    // Keep squaring x while one
    // of the following doesn't
    // happen
    // (i) d does not reach n-1
    // (ii) (x^2) % n is not 1
    // (iii) (x^2) % n is not n-1
    while (d != n-1n)
    {
        x = (x * x) % n;
        d *= 2n;

        if (x == 1n)	
            return false;
        if (x == n-1n)
            return true;
    }

    // Return composite
    return false;
}

// It returns false if n is
// composite and returns true if n
// is probably prime. k is an
// input parameter that determines
// accuracy level. Higher value of
// k indicates more accuracy.
function isPrime(n, k=14)
{
    // (JML- all literal integers converted to use n suffix denoting BigInt)
    // Corner cases
    if (n <= 1n || n == 4n) return false;
    if (n <= 3n) return true;

    // Find r such that n =
    // 2^d * r + 1 for some r >= 1
    let d = n - 1n;
    let s = 0;
    while (d % 2n == 0n) {
        d /= 2n;
        s++;
    }

    // Iterate given nber of 'k' times
    for (let i = 0; i < k; i++)
        if (!millerTest(d, n))
            return [false];

    return [true, d, s];
}