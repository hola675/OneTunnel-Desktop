# Future native Windows capture evaluation

**Status: deferred. M2-R is not a system VPN.**

M1/M2-R use Xray's local SOCKS inbound and selectively route a chosen process
through external Proxifier rules. They do not create a Wintun adapter, run
tun2socks, capture all Windows traffic, or change routes, DNS, IPv6, firewall,
or adapters. Administrator privileges are not required.

The previous Wintun/tun2socks snapshot, rollback, watchdog, and route planner
are preserved externally for **M3 — Native Windows Capture Evaluation**. This
document records evaluation requirements only; it does not authorize running
that prototype.

## Questions for M3

- Can a complete pre-change snapshot and idempotent rollback be proven?
- How will IPv4, IPv6, DNS, UDP, and physical proxy reachability be handled?
- What fail-closed behavior prevents leaks during connect, disconnect, and
  process crashes?
- Which route and adapter mutations are strictly required, and can each be
  reliably reversed?
- What tests can validate the design without affecting the host's network?

No routing plan, DNS policy, IPv6 policy, or Wintun runtime execution is part
of M2-R. Revisit those topics only after a separate M3 evaluation is approved
and designed.
