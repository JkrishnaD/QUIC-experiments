# QUIC Connections

Before talking about streams, multiplexing, or performance,
we must understand what a **QUIC connection** actually is.
Most misunderstandings about QUIC come from assuming:
"a connection is just a socket".
In QUIC, that assumption is wrong.

---

## What Is a QUIC Connection?

A QUIC connection is:
- A long-lived, encrypted communication context
- Identified by **Connection IDs**, not IP + port
- Capable of surviving network changes
- The container for all streams

It is closer to a **session** than a socket.

---

## TCP Connection vs QUIC Connection

### TCP
A TCP connection is identified by a **5-tuple**:
- (src_ip, src_port, dst_ip, dst_port, protocol)

If **any** of these change:
→ the connection breaks.

This is why:
- Wi-Fi → mobile data transitions break connections
- NAT rebinding causes resets
- Mobility is painful

---

### QUIC
A QUIC connection is identified by:
- **Connection ID (CID)**

The CID is carried inside QUIC packets (which are themselves inside UDP).

IP and port are treated as **routing metadata**, not identity.

**Important**: The connection is still transported over UDP, but UDP is just the carrier—the connection identity lives at the QUIC layer.

---

## Mental Model
```
TCP:
(IP, Port) tuple ───────────▶ Connection identity
                               (change = break)

QUIC:
Connection ID ──────────────▶ Connection identity
      ▲
      └─── IP/Port can change freely
```

This single design decision enables:
- Connection migration
- NAT rebinding survival
- Mobile-friendly networking

---

## Handshake and Encryption (High-Level)

QUIC **integrates TLS 1.3 into the transport layer**.

Important consequences:
- Encryption is mandatory (no plaintext mode exists)
- Handshake and transport are tightly coupled
- You never "add TLS on top of QUIC"—TLS **is part of** QUIC

This is fundamentally different from TCP + TLS, where they are separate layers.

---

## What Happens During a QUIC Handshake?

At a high level:

1. **Client sends Initial packet** (contains ClientHello)
2. **Server responds** with Initial and Handshake packets
3. **Cryptographic keys are derived** (via TLS 1.3 key schedule)
4. **Connection transitions to encrypted mode**
5. **Application data can begin flowing**

There is no separate:
- TCP 3-way handshake
- then separate TLS handshake

**Result**: QUIC typically completes connection establishment in **1-RTT** (or even 0-RTT with session resumption), compared to TCP+TLS's 2-3 RTTs.

This reduces latency significantly.

---

## Connection Lifetime

A QUIC connection:
- Can stay alive for hours or days
- Can carry many independent streams (up to 2^62 - 1)
- Can outlive individual network paths (via migration)
- Is kept alive through periodic packets (even when idle)

**Closing a connection**:
- Closes all active streams immediately
- Invalidates all connection state
- Can be graceful (CONNECTION_CLOSE frame) or immediate

Streams come and go within the connection.
The connection remains stable.

---

## Connection Migration

One of QUIC's most powerful features:

**Scenario**: You're on Wi-Fi, then switch to mobile data.

**TCP**: Connection breaks. Must reconnect. Lost state.

**QUIC**: 
1. Your IP/port changes
2. Client sends packet with same Connection ID from new address
3. Server validates the new path
4. Connection continues seamlessly

This happens **transparently** to the application layer.

---

## Why This Matters Before Streams

Every stream you create:
- Lives **inside a connection**
- Shares the connection's congestion control
- Shares connection-level flow control
- Inherits the connection's encryption context

If you don't understand the connection,
streams will feel magical and confusing.

**Key insight**: The connection provides the foundation; streams provide the multiplexing.

---

## Key Observations

<img width="824" height="178" alt="image" src="https://github.com/user-attachments/assets/b325e69a-3b9d-4534-997d-af7799d0f057" />


*Figure: A QUIC connection's lifecycle showing establishment*
- A QUIC connection can be fully established without creating any streams.
- Connection establishment occurs before any application-level data exchange.
- TLS encryption is mandatory and implicit; no plaintext phase exists.
- Closing the connection on the client propagates cleanly to the server.
- Streams are not required for connection lifecycle events.

---

## Key Takeaways

- QUIC connections are **not socket-bound** (UDP is just transport)
- Connection identity is abstracted via Connection IDs
- Encryption is mandatory and built-in (TLS 1.3 integrated)
- The connection is the **unit of continuity**
- Connections can migrate across network changes
- Streams are **multiplexed within** the connection

Next, we will explore **streams**, the real power of QUIC.
