const fs = require('fs');
const { AbiCoder, toBeHex } = require('ethers');

const encoder = new AbiCoder();

const outputTest = (values) => {
    process.stdout.write(encoder.encode(
        ['(bytes,bytes,uint256)[]'],
        [
            values.map(({ n, d, s }) => [
                toBeHex(BigInt(n.toString())),
                toBeHex(BigInt(d.toString())),
                s,
            ])
        ])
    );
};

const type = process.argv[2];
const tests = JSON.parse(fs.readFileSync(`./test/tests.json`, { encoding: 'utf-8' }))[type];
outputTest(tests);