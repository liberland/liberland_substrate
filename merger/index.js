const { execSync: execSyncInternal } = require('child_process');
const { sync } = require("glob");
const { commitId, newBranch, currentBranch } = require("./config.json");

const execSync = (command) => execSyncInternal(command, {
    stdio: 'inherit',
});

const mergePaths = [
    "substrate/bin/node/cli",
    "substrate/bin/node/rpc",
    "substrate/bin/node/runtime",
    "substrate/frame/democracy",
    "substrate/frame/elections-phragmen",
    "substrate/frame/identity",
    "substrate/frame/nfts",
    "substrate/frame/nfts/runtime-api",
    "substrate/frame/staking",
    "substrate/frame/staking/reward-curve",
    "substrate/frame/staking/reward-fn",
];

const startup = [
    "git remote add polkadot-sdk-upstream https://github.com/paritytech/polkadot-sdk.git",
    "git remote update",
    "cd ./polkadot-sdk",
    "git fetch origin",
    `git checkout ${commitId}`,
];

execSync(startup.join("; "));

const theirs = mergePaths.reduce((theirs, their) => {
    theirs.push(...sync(`${__dirname}/polkadot-sdk/${their}/**/*`));
    return theirs;
}, []).map((their) => their.replace(`${__dirname}/polkadot-sdk/`, "")).reduce(
    (theirs, path) => {
        theirs[path] = true;
        return theirs;
    },
    {},
);

try {
    execSync(`git merge --allow-unrelated-histories --no-commit polkadot-sdk-upstream/${newBranch};`);
} catch {}

const diffed = execSync("git diff --name-only").toString("utf-8").split("\n");
diffed.forEach(diff => {
    if (!theirs[diff]) {
        execSync(`git restore --staged ${diff}`);
    }
})

execSync(`git restore .`);
