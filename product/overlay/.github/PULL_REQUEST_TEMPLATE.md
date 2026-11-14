## What and why
<!-- the change, and the issue it closes (#…) -->

## How it was checked
- [ ] `make test`
- [ ] `make lint` (includes the dependency direction and the generated client types)
- [ ] `make conformance` (39/39)
- [ ] a test that fails without the change (for a bug fix)

## Rules
- [ ] no identifier, metadata, time stamp or randomness written into Markdown files or identities
- [ ] no change to `spec/SPEC.md` §1–§6 or the conformance fixtures
- [ ] new dependencies pinned to exact versions at least 14 days old
- [ ] no secret in the diff

This repository is generated from the development repository: an accepted pull request is carried there with your authorship and comes back with the next release (see CONTRIBUTING.md).
