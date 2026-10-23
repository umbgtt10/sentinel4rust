# sentinel4rust

Sentinels hunt mutants.

`sentinel4rust` will run on a dedicated machine, around the clock: it lists an owner's repositories on GitHub,
and in each one that opts in with a `sentinel4rust.toml` at its root, naming the crates to check, whenever `main`
moves it runs full mutation testing with [cargo-mutants](https://mutants.rs) on those crates at that exact commit,
then publishes the surviving mutants, with the commit hash, on the repository's own `mutants` branch. It is not a
gate: nothing waits for it, and it never touches `main`.

```toml
# sentinel4rust.toml
crates = ["my-crate", "my-crate-core"]
```

## Status

Under construction. Version `0.0.1` only reserves the name; it does nothing yet.

## License

MIT -- see [LICENSE](LICENSE).
