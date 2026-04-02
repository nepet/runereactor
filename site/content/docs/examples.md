+++
title = "Example Policies"
template = "page.html"
weight = 2
+++

# Example Policies

Each example below includes an interactive editor — modify the policy and see the compiled output update in real-time.

## Basic Operator Policy

A simple policy that restricts a rune to read-only balance queries and payments up to a certain amount.

<rf-playground format="json" source="allow methods: listfunds, xpay

when xpay:
  pnameamount_msat < 1000000001 or pnameamount_msat !"></rf-playground>

This policy:
- Allows only `listfunds` and `xpay` methods
- When calling `xpay`, the payment amount must be less than 1,000,000,001 millisatoshis (about 10,000 sats) — or the amount field must be absent

## Read-Only Access

Grant a rune that can only call read-only methods, with metadata tags for auditing.

<rf-playground format="cln" source="tag: purpose monitoring
tag: version 1

allow methods: listfunds, listpeerchannels, listnodes, listchannels"></rf-playground>

This policy:
- Tags the rune with a purpose and version (visible in `showrunes` output)
- Restricts to four read-only methods — the holder cannot modify any node state

## Advanced Policy with Rate Limiting

A more complex policy that combines peer restriction, method whitelisting, conditional constraints, and global rate limiting.

<rf-playground format="raw" source="id: 024b9a1fa8e006f1e3937f65f66c408e6da8e1ca728ea43222a7381df1cc449605

tag: purpose channel-management
tag: version 1

allow methods: listfunds, listpeerchannels, fundchannel, close

when fundchannel:
  pnameamount < 1000001

when close:
  pnamedestination = bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh

global:
  rate = 10"></rf-playground>

This policy:
- Restricts the rune to a specific commando peer (by node public key)
- Allows four methods: two read-only and two that modify state
- Limits `fundchannel` amounts to at most 1,000,000 sats
- Requires `close` to send funds to a specific cold wallet address
- Applies a global rate limit of 10 uses per minute across all methods
