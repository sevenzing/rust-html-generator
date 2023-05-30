small-fast *args:
    just generate "/Users/levlymarenko/innopolis/thesis/test-rust-crate/" "output.html" {{args}}

small-whole *args:
    just generate "/Users/levlymarenko/innopolis/thesis/test-rust-crate/" "output.html" -s {{args}}

big-fast *args:
    just generate "/Users/levlymarenko/innopolis/thesis/rust-ast/" "output_rust_ast.html" {{args}}

big-whole *args:
    just generate "/Users/levlymarenko/innopolis/thesis/rust-ast/" "output_rust_ast.html" -s {{args}}

generate path output *args:
    cargo run -- --dir {{path}} --output {{output}} {{args}}


analyze path name:
    echo "project size:" `du -I target -I .git -h -s {{path}} | awk '{ print $1 }'`
    echo "project lines:"
    cloc --exclude-dir target {{path}} 2> /dev/null
    #time just generate {{path}} ./tmp-{{name}}.html > /dev/null
    time just generate {{path}} ./tmp-{{name}}.html -s
    wc -l ./tmp-{{name}}.html
    echo "report size:" `du -I target -I .git -h -s ./tmp-{{name}}.html | awk '{ print $1 }'`
