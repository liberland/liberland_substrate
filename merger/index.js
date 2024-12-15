const { execSync: execSyncInternal } = require('child_process');
const { sync } = require("glob");
const { newBranch } = require("./config.json");

const execSync = (command) => execSyncInternal(command, {
    stdio: 'inherit',
});
const execOutput = (command) => execSyncInternal(command, {
    encoding: "utf-8",
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
    `cd ${__dirname}/polkadot-sdk`,
    "git fetch origin",
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

const staged = execOutput("git diff --staged --name-only").split("\n").filter(Boolean);
staged.forEach(diff => {
    if (!theirs[diff]) {
        try {
            execSync(`git restore --staged ${diff}`);
            execSync(`git restore ${diff}`);
        } catch {
            execSync(`rm -rf ${diff}`);
        }
    }
});

execSync("git status | grep 'deleted by us' | sed 's/deleted by us: //' | xargs git rm");