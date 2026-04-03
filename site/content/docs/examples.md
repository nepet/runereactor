+++
title = "Example Policies"
template = "page.html"
weight = 2
+++

# Example Policies

Each example below includes an interactive editor — modify the policy and see the compiled output update in real-time. They are ordered from simplest to most complex.

## Monitoring Bot

Read-only access using prefix matching, with a deny rule for sensitive data.

<rf-playground minimal format="json" source="# Read-only access for a monitoring bot&#10;allow methods: ^list, ^get, summary&#10;&#10;# Deny listdatastore — stores sensitive data&#10;global:&#10;  method / listdatastore"></rf-playground>

This policy:
- Allows any method starting with `list` or `get`, plus `summary` — using the `^` prefix operator
- Denies `listdatastore` via a global restriction (it contains sensitive data)
- Two restrictions work together: the allow (OR'd alternatives) and the deny (AND'd separately)

## Payment App

A spending-limited rune for an app that can check balances and send payments.

<rf-playground minimal format="json" source="allow methods: listfunds, getinfo, xpay&#10;&#10;when xpay:&#10;  pnameamount_msat < 100000000 or pnameamount_msat !&#10;  rate = 10"></rf-playground>

This policy:
- Allows `listfunds` and `getinfo` for balance checks, plus `xpay` for payments
- Caps `xpay` payments at 100,000,000 msat (~1000 sats) — or allows invoice-embedded amounts (`!` = field absent)
- Rate limits `xpay` to 10 calls per minute

## Channel Operator

A full policy combining peer restriction, tags, prefix matching, conditional constraints with grouping, and a global rate limit.

<rf-playground minimal format="raw" source="id: 024b9a1fa8e006f1e3937f65f66c408e6da8e1ca728ea43222a7381df1cc449605&#10;&#10;tag: role channel-operator&#10;tag: version 1&#10;&#10;allow methods: ^list, getinfo, fundchannel, close, xpay&#10;&#10;when fundchannel:&#10;  pnameamount < 1000001&#10;&#10;when xpay:&#10;  (pnameamount_msat < 100000000 or pnameamount_msat !) and rate = 10&#10;&#10;when close:&#10;  pnamedestination = bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh&#10;&#10;global:&#10;  per = 1min"></rf-playground>

This policy:
- Locked to a specific commando peer by node public key
- Tagged with role and version for auditing (visible in `showrunes` output)
- Uses `^list` prefix for all read methods, plus specific write methods
- Limits `fundchannel` to 1,000,000 sats
- Caps `xpay` at ~1000 sats with rate limiting — using parenthesized grouping `(... or ...) and ...`
- Forces `close` to send to a specific cold wallet address
- Global rate limit of 1 call per minute across all methods
