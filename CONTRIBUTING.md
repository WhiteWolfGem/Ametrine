# Contributing to Ametrine

Thank you for your interest in contributing to Ametrine! To maintain a high standard of code quality and ensure the project's longevity, please follow these guidelines.

---

## Contributor License Agreement (CLA)

By submitting a contribution to Ametrine, you agree to the following:

1. Your contribution is licensed under the project's current **AGPL-3.0-only** license.
2. You grant the maintainer a non-exclusive, perpetual, irrevocable, worldwide, royalty-free license to use, modify, and re-license your contribution under any other license, including commercial or proprietary licenses.

This agreement allows the project to remain open-source while providing the maintainer the flexibility to offer hosted or "Pro" versions in the future.

---

## Technical Standards

1. **Rust Environment:** - Ensure you are using the latest stable version of Rust.
    - Run `cargo fmt` to maintain consistent formatting.
    - Run `cargo deny check` locally to ensure no incompatible licenses or security vulnerabilities are introduced.
2. **Database:** - Ametrine uses **sqlx** for PostgreSQL interactions. Please provide migrations for any schema changes.
3. **Pull Requests:** - Keep PRs focused. One feature or fix per PR is preferred.
    - Provide a clear description of the change and any relevant issue numbers.

---

## Communication

For significant changes or architectural shifts, please open a **GitHub Issue** (or a ticket on our internal Forgejo instance) to discuss your ideas before starting work.
