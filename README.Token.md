# Token notes on Embassy maintenance

## Token SOP for Embassy merges

### Background

Embassy upstream is quite bad at updating crate versions when the versions specified in their respective `Cargo.toml` files changes. For example, it is very common to see a crate with its version listed as X, and a dependency gets bumped from N to N+1, but the version number is changed. **This does not result in the same crate.**

This results in an issue whereby we will publish a crate to Artifactory, overwriting the previous version, which:

1. Causes issues with lockfiles having the SHAs change and getting annoyed.
1. Ends up building a different version than before the most recent merge.

Neither of which are good.

### Policy

To correct this, workflow should proceed as follows:

1. Sync with upstream `main` on GitHub.
1. Make sure to `git fetch` / `git pull` / etc. everything.
1. Create a feature branch for the merge off of `token-main`.
1. Merge `main` into the feature branch, resolving conflicts accordingly.
    1. Version numbers will surely conflict. Resolve this conflict by taking upstream's version number, multiplying the minor version it by 1000, then incrementing it by 1. If this number would be less than our current version, just use ours.
1. Make another commit which **touches every single `Cargo.toml` file**:
    1. Make sure all use the versioning scheme described above.
    1. Then increment the patch number by 1.
        1. So we should **never** see an embassy dependency with a patch number of 0 (quick visual sanity check).
    1. Then go through all the dependencies and update their version numbers to reflect the above.
    1. Commit the result.