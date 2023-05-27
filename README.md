# `scm`

`scm` is an ergonomic CLI tool to manage ScyllaDB clusters written in Rust. It's
very basic, very fast, and very easy to use.

## Installation

```bash
cargo install scm-cli
```

## Usage

```bash
scm --help
scm env create # create a new environment at dev.scm.toml
scm create "bob migration" # create a new migration
# edit the migration file in migrations folder...
scm apply # apply the migration to the default dev environment
```
