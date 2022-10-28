# Testing rust code parsing

`Syn` is a parsing library for parsing a stream of Rust tokens into a syntax tree of Rust source code.

Examples of usage:

## 1. Hello world

+ Rust source code:

```rust
fn main() {
    println!("Hello world");
}
```

+ Result syntax tree:

<details><summary>CLICK TO SHOW</summary>


```rust
File {
    shebang: None,
    attrs: [],
    items: [
        Fn(
            ItemFn {
                attrs: [],
                vis: Inherited,
                sig: Signature {
                    constness: None,
                    asyncness: None,
                    unsafety: None,
                    abi: None,
                    fn_token: Fn,
                    ident: Ident {
                        sym: main,
                        span: bytes(4..8),
                    },
                    generics: Generics {
                        lt_token: None,
                        params: [],
                        gt_token: None,
                        where_clause: None,
                    },
                    paren_token: Paren,
                    inputs: [],
                    variadic: None,
                    output: Default,
                },
                block: Block {
                    brace_token: Brace,
                    stmts: [
                        Semi(
                            Macro(
                                ExprMacro {
                                    attrs: [],
                                    mac: Macro {
                                        path: Path {
                                            leading_colon: None,
                                            segments: [
                                                PathSegment {
                                                    ident: Ident {
                                                        sym: println,
                                                        span: bytes(17..24),
                                                    },
                                                    arguments: None,
                                                },
                                            ],
                                        },
                                        bang_token: Bang,
                                        delimiter: Paren(
                                            Paren,
                                        ),
                                        tokens: TokenStream [
                                            Literal {
                                                lit: "Hello world",
                                                span: bytes(26..39),
                                            },
                                        ],
                                    },
                                },
                            ),
                            Semi,
                        ),
                    ],
                },
            },
        ),
    ],
}
```
</details>


## 2. Function definition

+ Rust source code:

```rust
fn plus_minus_mul(a: i32, b: i32, c: i32) -> i32 {
    a + b * c
}

fn main() {
    let a = plus_minus_mul(1, 2, 3);
}
```

+ Result syntax tree:

<details><summary>CLICK TO SHOW</summary>


```rust
File {
    shebang: None,
    attrs: [],
    items: [
        Fn(
            ItemFn {
                attrs: [],
                vis: Inherited,
                sig: Signature {
                    constness: None,
                    asyncness: None,
                    unsafety: None,
                    abi: None,
                    fn_token: Fn,
                    ident: Ident {
                        sym: plus_minus_mul,
                        span: bytes(4..18),
                    },
                    generics: Generics {
                        lt_token: None,
                        params: [],
                        gt_token: None,
                        where_clause: None,
                    },
                    paren_token: Paren,
                    inputs: [
                        Typed(
                            PatType {
                                attrs: [],
                                pat: Ident(
                                    PatIdent {
                                        attrs: [],
                                        by_ref: None,
                                        mutability: None,
                                        ident: Ident {
                                            sym: a,
                                            span: bytes(19..20),
                                        },
                                        subpat: None,
                                    },
                                ),
                                colon_token: Colon,
                                ty: Path(
                                    TypePath {
                                        qself: None,
                                        path: Path {
                                            leading_colon: None,
                                            segments: [
                                                PathSegment {
                                                    ident: Ident {
                                                        sym: i32,
                                                        span: bytes(22..25),
                                                    },
                                                    arguments: None,
                                                },
                                            ],
                                        },
                                    },
                                ),
                            },
                        ),
                        Comma,
                        Typed(
                            PatType {
                                attrs: [],
                                pat: Ident(
                                    PatIdent {
                                        attrs: [],
                                        by_ref: None,
                                        mutability: None,
                                        ident: Ident {
                                            sym: b,
                                            span: bytes(27..28),
                                        },
                                        subpat: None,
                                    },
                                ),
                                colon_token: Colon,
                                ty: Path(
                                    TypePath {
                                        qself: None,
                                        path: Path {
                                            leading_colon: None,
                                            segments: [
                                                PathSegment {
                                                    ident: Ident {
                                                        sym: i32,
                                                        span: bytes(30..33),
                                                    },
                                                    arguments: None,
                                                },
                                            ],
                                        },
                                    },
                                ),
                            },
                        ),
                        Comma,
                        Typed(
                            PatType {
                                attrs: [],
                                pat: Ident(
                                    PatIdent {
                                        attrs: [],
                                        by_ref: None,
                                        mutability: None,
                                        ident: Ident {
                                            sym: c,
                                            span: bytes(35..36),
                                        },
                                        subpat: None,
                                    },
                                ),
                                colon_token: Colon,
                                ty: Path(
                                    TypePath {
                                        qself: None,
                                        path: Path {
                                            leading_colon: None,
                                            segments: [
                                                PathSegment {
                                                    ident: Ident {
                                                        sym: i32,
                                                        span: bytes(38..41),
                                                    },
                                                    arguments: None,
                                                },
                                            ],
                                        },
                                    },
                                ),
                            },
                        ),
                    ],
                    variadic: None,
                    output: Type(
                        RArrow,
                        Path(
                            TypePath {
                                qself: None,
                                path: Path {
                                    leading_colon: None,
                                    segments: [
                                        PathSegment {
                                            ident: Ident {
                                                sym: i32,
                                                span: bytes(46..49),
                                            },
                                            arguments: None,
                                        },
                                    ],
                                },
                            },
                        ),
                    ),
                },
                block: Block {
                    brace_token: Brace,
                    stmts: [
                        Expr(
                            Binary(
                                ExprBinary {
                                    attrs: [],
                                    left: Path(
                                        ExprPath {
                                            attrs: [],
                                            qself: None,
                                            path: Path {
                                                leading_colon: None,
                                                segments: [
                                                    PathSegment {
                                                        ident: Ident {
                                                            sym: a,
                                                            span: bytes(56..57),
                                                        },
                                                        arguments: None,
                                                    },
                                                ],
                                            },
                                        },
                                    ),
                                    op: Add(
                                        Add,
                                    ),
                                    right: Binary(
                                        ExprBinary {
                                            attrs: [],
                                            left: Path(
                                                ExprPath {
                                                    attrs: [],
                                                    qself: None,
                                                    path: Path {
                                                        leading_colon: None,
                                                        segments: [
                                                            PathSegment {
                                                                ident: Ident {
                                                                    sym: b,
                                                                    span: bytes(60..61),
                                                                },
                                                                arguments: None,
                                                            },
                                                        ],
                                                    },
                                                },
                                            ),
                                            op: Mul(
                                                Star,
                                            ),
                                            right: Path(
                                                ExprPath {
                                                    attrs: [],
                                                    qself: None,
                                                    path: Path {
                                                        leading_colon: None,
                                                        segments: [
                                                            PathSegment {
                                                                ident: Ident {
                                                                    sym: c,
                                                                    span: bytes(64..65),
                                                                },
                                                                arguments: None,
                                                            },
                                                        ],
                                                    },
                                                },
                                            ),
                                        },
                                    ),
                                },
                            ),
                        ),
                    ],
                },
            },
        ),
        Fn(
            ItemFn {
                attrs: [],
                vis: Inherited,
                sig: Signature {
                    constness: None,
                    asyncness: None,
                    unsafety: None,
                    abi: None,
                    fn_token: Fn,
                    ident: Ident {
                        sym: main,
                        span: bytes(72..76),
                    },
                    generics: Generics {
                        lt_token: None,
                        params: [],
                        gt_token: None,
                        where_clause: None,
                    },
                    paren_token: Paren,
                    inputs: [],
                    variadic: None,
                    output: Default,
                },
                block: Block {
                    brace_token: Brace,
                    stmts: [
                        Local(
                            Local {
                                attrs: [],
                                let_token: Let,
                                pat: Ident(
                                    PatIdent {
                                        attrs: [],
                                        by_ref: None,
                                        mutability: None,
                                        ident: Ident {
                                            sym: a,
                                            span: bytes(89..90),
                                        },
                                        subpat: None,
                                    },
                                ),
                                init: Some(
                                    (
                                        Eq,
                                        Call(
                                            ExprCall {
                                                attrs: [],
                                                func: Path(
                                                    ExprPath {
                                                        attrs: [],
                                                        qself: None,
                                                        path: Path {
                                                            leading_colon: None,
                                                            segments: [
                                                                PathSegment {
                                                                    ident: Ident {
                                                                        sym: plus_minus_mul,
                                                                        span: bytes(93..107),
                                                                    },
                                                                    arguments: None,
                                                                },
                                                            ],
                                                        },
                                                    },
                                                ),
                                                paren_token: Paren,
                                                args: [
                                                    Lit(
                                                        ExprLit {
                                                            attrs: [],
                                                            lit: Int(
                                                                LitInt {
                                                                    token: 1,
                                                                },
                                                            ),
                                                        },
                                                    ),
                                                    Comma,
                                                    Lit(
                                                        ExprLit {
                                                            attrs: [],
                                                            lit: Int(
                                                                LitInt {
                                                                    token: 2,
                                                                },
                                                            ),
                                                        },
                                                    ),
                                                    Comma,
                                                    Lit(
                                                        ExprLit {
                                                            attrs: [],
                                                            lit: Int(
                                                                LitInt {
                                                                    token: 3,
                                                                },
                                                            ),
                                                        },
                                                    ),
                                                ],
                                            },
                                        ),
                                    ),
                                ),
                                semi_token: Semi,
                            },
                        ),
                    ],
                },
            },
        ),
    ],
}
```
</details>


