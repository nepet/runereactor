+++
title = "Syntax Reference"
template = "page.html"
weight = 1
+++

# .rf Syntax Reference

A `.rf` policy file consists of **directives** — top-level statements that define what a rune allows. The compiler transforms these directives into rune restrictions (arrays of conditions in Conjunctive Normal Form).

## Directives

| Directive | Purpose | Example |
|-----------|---------|---------|
| `id: hex` | Restrict to a specific commando peer | `id: 024b9a1f...` |
| `tag: field value` | Attach metadata to the rune (compiles to a comment restriction) | `tag: purpose payments` |
| `allow methods: a, b, c` | Whitelist specific methods | `allow methods: listfunds, xpay` |
| `when method:` | Apply constraints only when a specific method is called | `when xpay:` |
| `global:` | Apply constraints to all method calls | `global:` |

### id

Restricts the rune to a specific commando peer by their Lightning node public key. Only that peer can use the rune.

<rf-playground format="raw" source="id: 024b9a1fa8e006f1e3937f65f66c408e6da8e1ca728ea43222a7381df1cc449605"></rf-playground>

### tag

Adds metadata as a comment restriction (`#` operator). Comment restrictions always pass — Core Lightning ignores them during authorization. They are visible in `lightning-cli showrunes` output.

<rf-playground format="raw" source="tag: purpose channel-management&#10;tag: version 2"></rf-playground>

### allow methods

Creates a method whitelist. The listed methods are the only ones the rune holder can call.

<rf-playground format="raw" source="allow methods: listfunds, listpeerchannels, xpay"></rf-playground>

### when

Applies constraints conditionally — only when a specific method is being called. The constraints are indented below the `when` line.

<rf-playground format="raw" source="when xpay:&#10;  pnameamount_msat < 1000000001"></rf-playground>

Uses **negation bypass**: compiles to `method/xpay|pnameamount_msat<1000000001`. If the method is *not* xpay, the restriction passes automatically. If it *is* xpay, the condition must hold.

### global

Applies constraints to every method call, regardless of which method is used. Indented below the `global:` line.

<rf-playground format="raw" source="global:&#10;  rate = 10"></rf-playground>

## Operators

All 11 rune condition operators are supported:

| Operator | Name | Meaning | Example |
|----------|------|---------|---------|
| `=` | Equal | Field equals value | `pnamedestination = bc1qaddr` |
| `/` | Not equal | Field does not equal value | `method / listdatastore` |
| `!` | Missing | Field is not present | `pnameamount_msat !` |
| `<` | Less than | Integer less than value | `pnameamount < 1000001` |
| `>` | Greater than | Integer greater than value | `pnameamount > 0` |
| `{` | Lexicographically less | String sorts before value | `version { 3.0` |
| `}` | Lexicographically greater | String sorts after value | `version } 1.0` |
| `^` | Starts with | Field value starts with string | `method ^ list` |
| `$` | Ends with | Field value ends with string | `pnamedest $ xyz` |
| `~` | Contains | Field value contains string | `pnamedesc ~ test` |
| `#` | Comment | Always passes (used for tags) | `purpose#payments` |

## Fields

Core Lightning checks these fields when evaluating rune restrictions:

### Built-in Fields

| Field | Description | Example |
|-------|-------------|---------|
| `time` | Current UNIX timestamp | `time < 1656759180` |
| `id` | Node ID of the peer using the rune | `id = 024b9a1fa8e...` |
| `method` | The command being run | `method = withdraw` |
| `per` | Rate limit interval. Supports suffixes: `msec`, `usec`, `nsec`, `sec` (default), `min`, `hour`, `day` | `per = 5sec` |
| `rate` | Rate limit per minute. `rate=60` is equivalent to `per=1sec` | `rate = 10` |
| `pnum` | Number of parameters passed to the command | `pnum < 2` |

### Parameter Fields

These are composable — you combine a prefix with a parameter name or position to form the full field name.

#### `pnameX` — Named parameter

`X` is the name of the parameter as defined by the CLN command. For example, the `xpay` command has a parameter called `amount_msat`, so you'd write `pnameamount_msat`.

```
when xpay:
  pnameamount_msat < 1000000001
when fundchannel:
  pnameamount < 1000001
when close:
  pnamedestination = bc1qexampleaddress
```

Common parameter names: `amount_msat`, `amount`, `destination`, `description`, `label`, `invstring`, `bolt11`, `channel_id`.

> **Note:** Prior to CLN v24.05, underscores had to be removed from parameter names (e.g. `pnameamountmsat` instead of `pnameamount_msat`). This is no longer required.

#### `parrN` — Positional parameter

`N` is the zero-indexed position of the parameter. `parr0` is the first parameter, `parr1` is the second, etc.

```
when withdraw:
  parr0 = bc1qexampleaddress
```

This is useful when you want to constrain a parameter by its position rather than its name.

#### `pinvX_N` — Invoice field extraction

Parses the parameter named `X` as a bolt11 or bolt12 invoice, then extracts subfield `N` for comparison. The restriction fails if the parameter is missing, doesn't parse as an invoice, or `N` is not a valid subfield.

Valid subfields: `amount`, `description`, `node`.

```
when xpay:
  pinvinvstring_amount < 1000000
  pinvinvstring_node = 024b9a1fa8e006f1e3937f65f66c408e6da8e1ca728ea43222a7381df1cc449605
```

## Expressions

Inside `when` and `global` blocks, each indented line is a **condition**. A condition consists of a field name, an operator, and (optionally) a value.

### Implicit AND

Multiple lines within a block are implicitly AND'd together:

<rf-playground format="raw" source="when xpay:&#10;  pnameamount_msat < 1000000001&#10;  rate = 10"></rf-playground>

This produces two separate restrictions — both must pass.

### Explicit OR

Use `or` to combine conditions as alternatives within a single restriction:

<rf-playground format="raw" source="when xpay:&#10;  pnameamount_msat < 1000000001 or pnameamount_msat !"></rf-playground>

This produces one restriction with two alternatives — either one can pass.

### Grouping with Parentheses

Use parentheses to group sub-expressions:

<rf-playground format="raw" source="when xpay:&#10;  (pnameamount_msat < 1000000001 or pnameamount_msat !) and rate = 10"></rf-playground>

### Expression Grammar

The full expression grammar:

```
expression  = or_expr
or_expr     = and_expr ("or" and_expr)*
and_expr    = atom ("and" atom)*
atom        = condition | "(" expression ")"
condition   = field operator [value]
```

Operator precedence: `and` binds tighter than `or`. Parentheses override precedence.

## CNF Compilation

Rune restrictions are structured as an **AND of ORs** (Conjunctive Normal Form). Each restriction is a list of alternatives — at least one alternative must pass. All restrictions must pass.

The compiler automatically converts arbitrary boolean expressions into CNF using the distributive law:

```
(A and B) or C  →  (A or C) and (B or C)
```

This means you can write natural boolean expressions and the compiler handles the transformation.

### Negation Bypass

When you use `when method:`, the compiler prepends a negation bypass to each restriction:

<rf-playground format="raw" source="when xpay:&#10;  pnameamount_msat < 1000000001"></rf-playground>

The `method/xpay` alternative means "if the method is NOT xpay, this restriction passes." This is how conditional constraints work in the rune restriction model — the condition only applies when the specified method is being called.
