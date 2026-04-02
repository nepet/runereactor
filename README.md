# rune-forge

Compile human-readable `.rf` policy files into [CLN rune](https://docs.corelightning.org/docs/runes) restrictions.

## What it does

Writing CLN rune restrictions by hand is tedious and error-prone, especially when you need conditional constraints, method whitelists, and rate limits. rune-forge lets you express authorization policies in a readable format and compiles them into the restriction arrays that `lightning-cli createrune` expects.

```
                  .rf policy
                      |
                  [rune-forge]
                      |
          +-----------+-----------+
          |           |           |
        JSON      createrune     raw
      (array)     (command)    (string)
```

## The `.rf` Policy Language

A policy file consists of directives:

```bash
# Restrict this rune to a specific peer
id: 024b9a1fa8e006f1e3937f65f66c408e6da8e1ca728ea43222a7381df1cc449605

# Metadata visible in showrunes output
tag: purpose channel-management
tag: version 1

allow methods: listfunds, listpeerchannels, fundchannel, close

# Fund channels up to 1,000,000 sats
when fundchannel:
  pnameamount < 1000001

# Only allow closing to a known cold wallet
when close:
  pnamedestination = bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh
```

### Directives

| Directive | Purpose | Example |
|-----------|---------|---------|
| `id: hex` | Restrict to a specific commando peer (see below) | `id: 024b9a1f...` |
| `tag: field value` | Attach metadata to the rune (see below) | `tag: purpose payments` |
| `allow methods: a, b, c` | Method whitelist | `allow methods: listfunds, xpay` |
| `when method:` | Conditional constraints for a method | `when xpay:` |
| `global:` | Constraints applied to all methods | `global:` |

### The `id` Directive

The `id` directive restricts a rune to a specific commando peer by their Lightning node ID. When set, only that peer can use the rune to execute commands. This compiles to an `id=<pubkey>` restriction that Core Lightning enforces directly.

```bash
id: 024b9a1fa8e006f1e3937f65f66c408e6da8e1ca728ea43222a7381df1cc449605
```

### Tags

Tags compile to comment restrictions (`#` operator) which always pass -- Core Lightning ignores them entirely during authorization. They are useful as metadata that travels with the rune and is visible in `lightning-cli showrunes` output.

A rune receiver can use tags for their own verification purposes. For example, a service receiving a rune could check that a `purpose` tag matches the expected use case, or that a `version` tag meets a minimum requirement, before accepting the rune -- even though Core Lightning itself does not enforce these.

```bash
tag: purpose channel-management
tag: issued_by admin-tool
tag: version 2
```

### Expressions

Inside `when` and `global` blocks, each indented line is a condition. Multiple lines are implicitly AND'd.

```bash
when xpay:
  pnameamount_msat < 1000000001    # these two lines
  rate = 10                         # are AND'd together
```

Use `or` for alternatives and parentheses for grouping:

```bash
when xpay:
  (pnameamount_msat < 1000000001 or pnameamount_msat !) and rate = 10
```

### Operators

All [rune condition operators](https://github.com/rustyrussell/runes) are supported:

| Operator | Meaning | Example |
|----------|---------|---------|
| `=` | Equals | `pnamedestination = bc1qaddr` |
| `/` | Not equal | `method / listdatastore` |
| `!` | Field is missing | `pnameamount_msat !` |
| `<` | Less than (integer) | `pnameamount < 1000001` |
| `>` | Greater than (integer) | `pnameamount > 0` |
| `{` | Lexicographically less than | `version { 3.0` |
| `}` | Lexicographically greater than | `version } 1.0` |
| `^` | Starts with | `method ^ list` |
| `$` | Ends with | `pnamedest $ xyz` |
| `~` | Contains | `pnamedesc ~ test` |
| `#` | Comment (always passes) | (used internally for tags) |

## Build

```bash
cargo build --release
```

## Usage

```bash
# JSON array-of-arrays (default)
rune-forge examples/operator.rf

# lightning-cli createrune command
rune-forge examples/operator.rf --format cln

# Raw restriction string
rune-forge examples/operator.rf --format raw

# Read from stdin
cat examples/operator.rf | rune-forge -
```

### Output formats

**JSON** (default) — ready for programmatic use or piping to other tools:

```bash
$ rune-forge policy.rf --format json
[["method=listfunds","method=xpay"],["method/xpay","pnameamount_msat<1000000001"]]
```

**CLN** — paste directly into your terminal:

```bash
$ rune-forge policy.rf --format cln
lightning-cli createrune -k "restrictions"='[["method=listfunds","method=xpay"],["method/xpay","pnameamount_msat<1000000001"]]'
```

**Raw** — the restriction string as used in the rune wire format (`|` separates alternatives, `&` separates restrictions):

```bash
$ rune-forge policy.rf --format raw
method=listfunds|method=xpay&method/xpay|pnameamount_msat<1000000001
```

## How It Works

rune-forge compiles `.rf` policies in two phases:

1. **Parse** — the `.rf` text is parsed into an AST of directives and expression trees
2. **Compile** — the AST is flattened into a `RunePolicy` (conjunction of restrictions in CNF)

Key transformations the compiler performs:

- **Method whitelist** — `allow methods: a, b, c` becomes one restriction: `method=a|method=b|method=c`
- **Negation bypass** — `when xpay: condition` becomes `method/xpay|condition` (if you're *not* calling xpay, the restriction passes; if you *are*, the condition must hold)
- **CNF conversion** — expressions like `(A and B) or C` are distributed into `(A or C) and (B or C)` to fit the rune restriction model (AND of ORs)

## Library Usage

rune-forge is also a library crate:

```rust
use rune_forge::parser::parse_policy;
use rune_forge::compiler::compile;

let input = r#"
allow methods: listfunds, xpay
when xpay:
  pnameamount_msat < 1000000001
"#;

let policy = parse_policy(input).unwrap();
let rune_policy = compile(&policy).unwrap();

for restriction in &rune_policy.restrictions {
    let alts: Vec<String> = restriction.alternatives.iter()
        .map(|c| format!("{}{}{}", c.field, c.op.as_char(), c.value))
        .collect();
    println!("{}", alts.join(" | "));
}
```

## Tests

```bash
make test
```

## Related Projects

- [futhark](https://github.com/nepet/futhark) — Rust implementation of the runes authorization library (`runeauth` crate)

## License

MIT
