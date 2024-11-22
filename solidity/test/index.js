import { ethers } from 'ethers';

const outputTest = ({ n,d,s }) => {
    process.stdout.write(ethers.utils.defaultAbiCoder.encode(
        ['bytes', 'bytes', 'uint256'],
        [
            ethers.BigNumber.from(n.toString()),
            ethers.BigNumber.from(d.toString()),
            s,
        ])
    );
};

const type = process.argv[2];
const index = proces.argv[3];
const tests = JSON.parse(fs.readFileSync("./tests.json", { encoding: "utf-8" }))[type];
if (index === "length") {
    process.stdout.write(ethers.utils.defaultAbiCoder.encode(['uint256'], [tests.length]));
} else {
    outputTest(tests[index]);
}