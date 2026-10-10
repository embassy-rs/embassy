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

## AI Policy

- Using AI / LLM to write code is **allowed**.
- You **must** disclose AI usage in the PR description.
- You **must** review and understand every single line of AI-written code before submitting the PR, and be prepared to answer questions and make fixes to it.
- Fully autonomous agents like OpenClaw are **NOT allowed**. Every issue, every PR must have a real human behind it.
- Using AI / LLM to write PR descriptions, commit messages, comments, or replies to reviewers is **NOT allowed**. You must write them yourself. AI must not replace human-to-human communication.
    - If you need help writing English, consider using a traditional translator like Google Translate instead of LLMs. They do a great job without the pitfalls of LLMs (overly verbose text, hallucinations, etc.).
- Low-effort pull requests are **NOT allowed**. Don't simply paste the issue text into an LLM and submit the result. It will most likely be a bad contribution. If fixing the issue was so easy, the person opening the issue or a maintainer would've already done it.
- Consider the time it takes to review your PRs. Maintainer time is the most scarce resource in the Embassy project. Keep diffs minimal, ask first before doing thousands-of-lines refactors, split PRs if you can. This has always been good practice, but it's especially relevant now that LLMs make it too easy to generate mountains of code.
- Pull requests not following this policy will be closed.
