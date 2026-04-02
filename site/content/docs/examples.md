+++
title = "Example Policies"
template = "page.html"
weight = 2
+++

# Example Policies

Each example below includes an interactive editor — modify the policy and see the compiled output update in real-time. They are ordered from simplest to most complex.

## Simple Method Whitelist

The most basic policy — just allow a few methods with no additional constraints.

<rf-playground format="json" source="allow methods: listfunds, listpeerchannels, getinfo"></rf-playground>

This policy:
- Allows only three read-only methods: `listfunds`, `listpeerchannels`, and `getinfo`
- No tags, no constraints — the simplest possible rune policy

## Read-Only Access with Tags

Grant a rune that can only call read-only methods, with metadata tags for auditing.

<rf-playground format="cln" source="tag: purpose monitoring&#10;tag: version 1&#10;&#10;allow methods: listfunds, listpeerchannels, listnodes, listchannels"></rf-playground>

This policy:
- Tags the rune with a purpose and version (visible in `showrunes` output)
- Restricts to four read-only methods — the holder cannot modify any node state

## Tagged Operator Policy

A policy with tags, method whitelisting, and conditional constraints on specific methods.

<rf-playground format="json" source="tag: operator_id default-operator&#10;&#10;allow methods: listfunds, listpeerchannels, fundchannel, close, invoice, xpay&#10;&#10;when fundchannel:&#10;  pnameamount < 1000001&#10;&#10;when xpay:&#10;  pnameamount_msat < 1000000001 or pnameamount_msat !"></rf-playground>

This policy:
- Tags the rune with an operator ID for auditing
- Allows six methods including both read-only and state-changing operations
- Limits `fundchannel` amounts to at most 1,000,000 sats
- When calling `xpay`, the payment amount must be under ~1 BTC — or the amount field must be absent (invoice-embedded amount)

## Advanced Policy with Rate Limiting

A more complex policy that combines peer restriction, method whitelisting, conditional constraints, and global rate limiting.

<rf-playground format="raw" source="id: 024b9a1fa8e006f1e3937f65f66c408e6da8e1ca728ea43222a7381df1cc449605&#10;&#10;tag: purpose channel-management&#10;tag: version 1&#10;&#10;allow methods: listfunds, listpeerchannels, fundchannel, close&#10;&#10;when fundchannel:&#10;  pnameamount < 1000001&#10;&#10;when close:&#10;  pnamedestination = bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh&#10;&#10;global:&#10;  rate = 10"></rf-playground>

This policy:
- Restricts the rune to a specific commando peer (by node public key)
- Allows four methods: two read-only and two that modify state
- Limits `fundchannel` amounts to at most 1,000,000 sats
- Requires `close` to send funds to a specific cold wallet address
- Applies a global rate limit of 10 uses per minute across all methods
