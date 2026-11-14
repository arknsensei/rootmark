# Contributing to RootMark

Thank you for looking. Issues and pull requests are welcome in this repository.

## How a change lands

This repository is **generated** from the development repository [arknsensei/rootmark](https://github.com/arknsensei/rootmark), one commit per release, and its `main` changes only through those transfer commits (`transfer-verify` checks that every file is what the development repository's tag produces). So:

1. **Open an issue** for a bug or an idea (templates in `.github/ISSUE_TEMPLATE/`).
2. **Open a pull request** here if you have a change. It is reviewed here; CI runs the same checks as for a release (`make test`, `make lint`, `make conformance`, dependency direction and age).
3. **An accepted pull request is carried into the development repository by a maintainer**, with your authorship (`Co-authored-by`), into the work of a round there. It comes back here with the next release's transfer commit, and your pull request is closed with a link to it. It is not merged here directly.

## Rules that do not bend

- **The Markdown files are the truth.** Nothing writes identifiers or metadata into a Markdown file. Identifiers carry no time stamp and no randomness.
- **No silent failure.** An unresolved reference, a missing file or a manifest mismatch is an error.
- **Languages.** Runtime code is Rust only (`core`, `index`, `server`, `bindings`); TypeScript only in `clients/`. Dependencies point one way: `core ← index ← server ← clients`.
- **Frozen surfaces.** `spec/SPEC.md` §1–§6 and the conformance fixtures change only with a specification version change.
- **Tests are not changed to make them pass.**
- **Pinned dependencies.** Exact versions, published at least 14 days before they are adopted (`scripts/check_dep_age.sh`, in CI).

## Building and testing

Rust 1.92.0 (`rust-toolchain.toml`), Node 24 for the clients.

```sh
make models          # the pinned embedder model (models/MANIFEST.json)
make test            # cargo test + the clients' tests
make lint            # fmt, clippy, the clients' tsc/eslint, dependency direction, generated client types
make conformance     # the specification's conformance suite, 39/39
```

By contributing you agree that your contribution is licensed under the Apache License 2.0 (LICENSE). Be kind: [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
