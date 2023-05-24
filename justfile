small-fast:
    cargo run -- --dir "/Users/levlymarenko/innopolis/thesis/test-rust-crate/" --project-name "test-rust-crate"

small-whole:
    cargo run -- --dir "/Users/levlymarenko/innopolis/thesis/test-rust-crate/" --project-name "test-rust-crate" -s

big-fast:
    cargo run -- --dir "/Users/levlymarenko/innopolis/thesis/rust-ast/" --project-name "rust-ast" --output "output_rust_ast.html"

big-whole:
    cargo run -- --dir "/Users/levlymarenko/innopolis/thesis/rust-ast/" --project-name "rust-ast" --output "output_rust_ast.html" -s
