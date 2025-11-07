# Github config and workflows

In this folder there is configuration for codecoverage, dependabot, and ci
workflows that check the library more deeply than the default configurations.

This folder can be or was merged using a --allow-unrelated-histories merge
strategy from <https://github.com/jonhoo/rust-ci-conf/> which provides a
reasonably sensible base for writing your own ci on. By using this strategy
the history of the CI repo is included in your repo, and future updates to
the CI can be merged later.

To perform this merge run:

```shell
git remote add ci https://github.com/jonhoo/rust-ci-conf.git
git fetch ci
git merge --allow-unrelated-histories ci/main
```

An overview of the files in this project is available at:
<https://www.youtube.com/watch?v=xUH-4y92jPg&t=491s>, which contains some
rationale for decisions and runs through an example of solving minimal version
and OpenSSL issues.
