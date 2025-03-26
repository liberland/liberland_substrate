# Deploying contracts to production

1. Open https://ui.use.ink/
2. Connect to network you want to deploy to
3. Click `Add New Contract` -> `Upload New Contract Code`
4. Select deployer account
5. Give some descriptive name (this will only be stored in local browser, not public)
6. Upload `*.contract` file from the `artifacts/` directory as Contract Bundle
7. Click `Next`
8. If required, provide arguments to the Constructor (for example, msig_court requires threshold and judges to be set at this step)
9. Click `Next` -> `Upload & Instantiate`
10. Update contract's README.md with the new address if applicable
