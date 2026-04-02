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

```
id: 024b9a1fa8e006f1e3937f65f66c408e6da8e1ca728ea43222a7381df1cc449605
```

Compiles to: `id=024b9a1fa8e006f1e3937f65f66c408e6da8e1ca728ea43222a7381df1cc449605`

### tag

Adds metadata as a comment restriction (`#` operator). Comment restrictions always pass — Core Lightning ignores them during authorization. They are visible in `lightning-cli showrunes` output.

```
tag: purpose channel-management
tag: version 2
```

Compiles to: `#purpose=channel-management`, `#version=2`

### allow methods

Creates a method whitelist. The listed methods are the only ones the rune holder can call.

```
allow methods: listfunds, listpeerchannels, xpay
```

Compiles to one restriction with alternatives: `method=listfunds|method=listpeerchannels|method=xpay`

### when

Applies constraints conditionally — only when a specific method is being called. The constraints are indented below the `when` line.

```
when xpay:
  pnameamount_msat < 1000000001
```

Uses **negation bypass**: compiles to `method/xpay|pnameamount_msat<1000000001`. If the method is *not* xpay, the restriction passes automatically. If it *is* xpay, the condition must hold.

### global

Applies constraints to every method call, regardless of which method is used. Indented below the `global:` line.

```
global:
  rate = 10
```

Compiles to: `rate=10`

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
| `#` | Comment | Always passes (used for tags) | `#purpose=payments` |

## Expressions

Inside `when` and `global` blocks, each indented line is a **condition**. A condition consists of a field name, an operator, and (optionally) a value.

### Implicit AND

Multiple lines within a block are implicitly AND'd together:

```
when xpay:
  pnameamount_msat < 1000000001
  rate = 10
```

This produces two separate restrictions — both must pass.

### Explicit OR

Use `or` to combine conditions as alternatives within a single restriction:

```
when xpay:
  pnameamount_msat < 1000000001 or pnameamount_msat !
```

This produces one restriction with two alternatives — either one can pass.

### Grouping with Parentheses

Use parentheses to group sub-expressions:

```
when xpay:
  (pnameamount_msat < 1000000001 or pnameamount_msat !) and rate = 10
```

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

```
when xpay:
  pnameamount_msat < 1000000001
```

Becomes: `method/xpay | pnameamount_msat<1000000001`

The `method/xpay` alternative means "if the method is NOT xpay, this restriction passes." This is how conditional constraints work in the rune restriction model — the condition only applies when the specified method is being called.
