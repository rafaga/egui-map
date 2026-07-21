# Contributing to egui-map

First of all: thank you for considering a contribution! 🎉 Whether it's a bug
report, a feature idea, a doc fix or a pull request, every bit of help is
welcome and appreciated.

This is a small project maintained in spare time, so please be patient with
responses — and be kind. We're all here to build something useful and learn
along the way.

## Ways to contribute

- **Report bugs**: open an issue describing what happened, what you expected,
  and how to reproduce it. A minimal code sample or screenshot helps a lot.
- **Suggest features**: open an issue first so we can discuss the idea before
  you invest time in an implementation.
- **Improve docs**: typos, unclear explanations, missing examples — docs PRs
  are always easy wins.
- **Send code**: fork the repo, create a branch, and open a pull request.

## Ground rules for pull requests

Please keep these in mind; they make reviews fast and friendly for everyone:

1. **One thing per PR.** Small, focused pull requests are much easier to
   review and merge than big ones.
2. **Keep the style.** Run the standard Rust tooling before submitting:
   ```sh
   cargo fmt
   cargo clippy
   ```
3. **Keep tests green.** Make sure the whole suite passes:
   ```sh
   cargo test
   ```
   If you add a feature, please add tests for it. If you fix a bug, a
   regression test is very welcome.
4. **Document public API.** New public items should have doc comments, and the
   docs must build without warnings:
   ```sh
   RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
   ```
5. **Don't break compatibility lightly.** This crate is published on
   crates.io; if your change alters the public API, mention it clearly in the
   PR description so we can discuss versioning.
6. **Write a clear PR description.** A couple of sentences about *what* and
   *why* saves everyone time. Screenshots or GIFs are gold for visual changes
   (this is a UI widget, after all!).

## Trying things out

You can run the bundled examples to see your changes in action:

```sh
cargo run --example basic
cargo run --example custom_template
cargo run --example svg_template
```

## License

By contributing, you agree that your contributions will be licensed under the
same [MIT license](LICENSE.md) that covers the project.

---

Thanks again for stopping by — we look forward to your ideas! 💜
