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
# edit the migration file
echo "CREATE KEYSPACE IF NOT EXISTS my_keyspace
WITH replication = {'class': 'SimpleStrategy', 'replication_factor': 1};" >> `scm list | tail -n 1`
scm apply # apply the migration to the default dev environment
```
