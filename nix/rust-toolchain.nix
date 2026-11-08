# Some nightlies lack clippy or rust-analyzer, so the newest one with every component is taken.
{ rust-bin }:
rust-bin.selectLatestNightlyWith (
  toolchain:
  toolchain.default.override {
    extensions = [
      "rust-src"
      "rust-analyzer"
      "clippy"
      "rustfmt"
      "llvm-tools"
    ];
  }
)
