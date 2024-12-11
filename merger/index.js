const fs = require("fs");
const { execSync } = require('child_process');
const { argv } = require("process");
const { sync } = require("glob");

const mergePaths = {
    // "our": "their"
    "substrate/bin/node/cli": "substrate/bin/node/cli",
    "substrate/bin/node/rpc": "substrate/bin/node/rpc",
    "substrate/bin/node/runtime": "substrate/bin/node/runtime",
    "substrate/frame/democracy": "substrate/frame/democracy",
    "substrate/frame/elections-phragmen": "substrate/frame/elections-phragmen",
    "substrate/frame/identity": "substrate/frame/identity",
    "substrate/frame/nfts": "substrate/frame/nfts",
    "substrate/frame/nfts/runtime-api": "substrate/frame/nfts/runtime-api",
    "substrate/frame/staking": "substrate/frame/staking",
    "substrate/frame/staking/reward-curve": "substrate/frame/staking/reward-curve",
    "substrate/frame/staking/reward-fn": "substrate/frame/staking/reward-fn",
};

const startup = [
    "cd ./polkadot-sdk",
    `git checkout ${argv[2]}`,
];

execSync(startup.join("; "), {
    stdio: "inherit",
});

Object.entries(mergePaths).forEach(([our, their]) => {
    const theirs = sync(`./polkadot-sdk/${their}/**/*`);
    theirs.forEach((file) => {
        execSync(`cp -rf ./${file} ../${our}`, { stdio: "inherit" })
    });
});