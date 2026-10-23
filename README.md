# sentinel4rust

Sentinels hunt mutants.

`sentinel4rust` will run on a dedicated machine, around the clock: it walks a list of repositories, and whenever
a repository's `main` moves it runs full mutation testing with [cargo-mutants](https://mutants.rs) against that
exact commit, then publishes the surviving mutants, with the commit hash, on the repository's own `mutants`
branch. It is not a gate: nothing waits for it, and it never touches `main`.

## Status

Under construction. Version `0.0.1` only reserves the name; it does nothing yet.

## License

MIT -- see [LICENSE](LICENSE).
