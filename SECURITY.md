# Security Policy

The escrow contract holds user funds. We take reports seriously and will work with you to fix and disclose them responsibly.

> **Status:** the contract is **unaudited** and deployed on **testnet only**. Do not use it with real funds. An external audit is tracked in the issue "Commission an external security audit of the escrow contract".

## Reporting a vulnerability

**Never open a public issue, discussion or pull request for a vulnerability.**

Report privately through either channel:

1. **GitHub Security Advisories (preferred):** [open a private advisory](https://github.com/zephyr-ramp/zephyr-contracts/security/advisories/new). Only maintainers can see it.
2. **Email:** the security contact listed on the [zephyr-ramp organization profile](https://github.com/zephyr-ramp).

Please include:

- the affected function(s) and commit or deployed contract ID;
- what an attacker can do (for example lock, steal or misroute funds, block refunds, or bypass auth);
- a reproduction, ideally a failing `soroban-sdk` test.

## What to expect

| Step | Target |
|---|---|
| Acknowledge your report | 3 business days |
| Initial assessment and severity | 7 days |
| Fix, or a mitigation plan, for critical issues | 30 days |

We'll keep you updated, credit you in the advisory unless you ask us not to, and agree a disclosure date with you. Please give us a reasonable window to ship a fix before disclosing publicly.

## Scope

In scope:

- `contracts/escrow`: any way to move escrowed funds other than the documented `claim`, `cancel` and `refund` paths, to block `refund`, or to bypass `require_auth`.
- `scripts/`: anything that could deploy or initialise a contract in an unsafe state.
- `bindings/`: generated client behaviour that could make a user sign something other than what they intended.

Out of scope: issues that need a compromised admin or anchor key (these are documented trust assumptions in the README's threat model), and testnet availability.

## Supported versions

Only the latest commit on `main` and the testnet deployment listed in the README are supported.
