# Contributor guide

## Pull requests

Embassy uses a FIFO [review queue](https://bot.embassy.dev/). Pull requests are automatically added to the queue when both of the following are true:

- The PR is marked as ready for review (not draft).
- The CI passes.

After opening a pull request, make sure to wait until CI passes or fails, and push more changes to fix CI if it doesn't. If you need help to get CI to pass, just ask on the pull request thread!

**If you have work-in-progress code** you're encouraged to open a draft PR. This lets other people know you're working on it. Marking the PR as draft ensures it doesn't get added to the review queue.

**If you want feedback on your work-in-progress code**, for example for API or design questions before implementing all the details, write a comment saying so in the PR, and mark it as ready for review. A design review is still a review!

## Formatting

The repo uses a few nightly-only `rustfmt` features, specified in [`rustfmt.toml`](./rustfmt.toml).

To get the CI `rustfmt` job to pass, do either of:

- Configure your IDE to format on save, and make sure it uses nightly rustfmt, for example by copying `rust-toolchain-nightly.toml` into `rust-toolchain.toml`.
- Run `fmtall.sh` before committing.
