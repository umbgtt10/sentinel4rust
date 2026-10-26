# sentinel4rust-core

The logic behind [`sentinel4rust`](https://github.com/umbgtt10/sentinel4rust): which repository to visit next and
when to back off, the roster, the record kept on each repository's `mutants` branch, and what git, cargo-mutants and
the run unit answer and what it means. It is `no_std` and depends on nothing; every effect -- git, cargo-mutants, the
clock, sleeping, the journal -- goes through a trait it declares. The service, the binary `sentinel4rust`, is the
`sentinel4rust` package.

What it does and why is in the [main README](https://github.com/umbgtt10/sentinel4rust#readme) and
[`docs/ARCHITECTURE.md`](https://github.com/umbgtt10/sentinel4rust/blob/main/docs/ARCHITECTURE.md).

MIT -- see [`LICENSE`](https://github.com/umbgtt10/sentinel4rust/blob/main/LICENSE).
