small-fast:
    cargo run --bin hl -- --dir "/Users/levlymarenko/innopolis/thesis/test-rust-crate/" --project-name "test-rust-crate"

small-whole:
    cargo run --bin hl -- --dir "/Users/levlymarenko/innopolis/thesis/test-rust-crate/" --project-name "test-rust-crate" -s

big-fast:
    cargo run --bin hl -- --dir "/Users/levlymarenko/innopolis/thesis/rust-ast/" --project-name "rust-ast" --output "output_rust_ast.html"

big-whole:
    cargo run --bin hl -- --dir "/Users/levlymarenko/innopolis/thesis/rust-ast/" --project-name "rust-ast" --output "output_rust_ast.html" -s
