# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->
## Unreleased - ReleaseDate

- `main`: abort instead of unwinding out of the generated `main`, which would leave dangling `'static` references to the executor on its stack frame ([#5277](https://github.com/embassy-rs/embassy/issues/5277)).

## 0.8.0 - 2026-03-12

- Update rust release id
- Add task name capability
- Follow executor api change
