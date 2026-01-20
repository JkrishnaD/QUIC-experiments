# Stream Starvation and Fairness in QUIC

## Overview

This document explores **stream starvation** in QUIC, a condition where competing streams on the same connection experience unfair resource allocation, causing some streams to be delayed or starved despite the protocol's multiplexing capabilities.

**Key insight**: Multiplexing enables concurrent streams, but does not guarantee fairness.

---

## Background

### What We Know So Far

From previous experiments, we've established that QUIC:

- ✓ Supports **multiple concurrent streams** on a single connection
- ✓ Maintains **independent ordering** per stream (no HOL blocking between streams)
- ✓ Applies **flow control** at both connection and stream levels
- ✓ Propagates **backpressure** naturally when receivers are slow
- ✓ Isolates **packet loss** to individual streams

### The Unanswered Question

However, one critical question remains:

> **When multiple streams compete for limited connection capacity, who gets to make progress?**

This is the question of **fairness**, and it's distinct from the question of **independence** that we've already validated.

---

## What Is Stream Starvation?

### Definition

**Stream starvation** occurs when one or more streams on a QUIC connection are prevented from making timely progress due to resource exhaustion caused by other streams on the same connection.

### Conditions for Starvation

Stream starvation can occur when:

1. **Multiple streams share a single QUIC connection**
2. **One stream aggressively sends large amounts of data** (bulk transfer)
3. **Other streams send smaller or latency-sensitive data** (interactive messages)
4. **The aggressive stream consumes most or all available flow-control credit**

### Observable Symptoms

When starvation occurs:

- **Delayed delivery** - Small messages wait behind bulk data
- **Increased write latency** - Send operations block waiting for credit
- **Bursty arrival** - Starved stream's data arrives in bursts rather than smoothly
- **Throughput imbalance** - Aggressive stream dominates bandwidth allocation

### Important Distinction

Stream starvation is **not**:

- A protocol error (both streams remain valid and open)
- A violation of QUIC specification
- The same as head-of-line blocking (data still arrives in order per stream)
- A connection failure (no errors occur)

Stream starvation is a **fairness problem**, not a correctness problem.

---

## Why Starvation Happens

### Connection-Level Flow Control

QUIC enforces flow control at two levels:

```
Connection Flow Control Window: 500 KB total
   ├─ Stream A: up to 500 KB (if available)
   └─ Stream B: up to 500 KB (if available)
```

**The problem**: Connection-level credit is **shared** among all streams.

If Stream A consumes all 500 KB of connection credit, Stream B cannot send anything until:
- Stream A's data is acknowledged, or
- The receiver increases the connection window

### No Built-In Scheduling

QUIC's transport layer:

- ✓ Provides multiplexing primitives
- ✓ Enforces flow control limits
- ✗ Does **not** implement inter-stream scheduling
- ✗ Does **not** guarantee fairness

The protocol assumes the **application layer** will handle fairness concerns.

### Aggressive Senders Win

Without application-level coordination:

```
Stream A (bulk):      ████████████████████████████ (100 MB file)
Stream B (messages):  ▌▌▌▌▌▌▌▌▌▌ (1 KB messages)

Connection credit: [████████████████████████████]
                    ↑ Stream A fills the entire window
                    
Stream B: Starved until Stream A yields credit
```

---

## Flow Control vs Fairness

### Critical Distinction

| Concern | Purpose | Guarantees |
|---------|---------|------------|
| **Flow Control** | Prevent receiver overload | Safety: memory limits, congestion control |
| **Fairness** | Equitable resource allocation | None at transport layer |

### What Flow Control Does

Flow control ensures:
- Receivers are not overwhelmed with data
- Memory buffers don't overflow
- Network congestion is managed
- Connections remain stable

### What Flow Control Does NOT Do

Flow control does **not** ensure:
- Equal bandwidth per stream
- Latency guarantees for any stream
- Priority-based resource allocation
- Protection against aggressive senders

### The Gap

```
┌──────────────────────────────────┐
│   Application Requirements       │
│   • Fairness                     │
│   • Priorities                   │
│   • Latency guarantees           │
└──────────────────────────────────┘
            ↕
      [GAP - Must be implemented by application]
            ↕
┌──────────────────────────────────┐
│   QUIC Transport Layer           │
│   • Flow control                 │
│   • Multiplexing                 │
│   • Reliability                  │
└──────────────────────────────────┘
```

**Fairness is a higher-level concern** that must be addressed by:
- Application logic
- Stream prioritization schemes (e.g., HTTP/3 priority signals)
- Scheduling policies
- Rate limiting per stream
- Explicit yielding or cooperative scheduling

---

## Experimental Hypothesis

### Setup

Create two streams on the same connection:

**Stream A (Bulk)**: 
- Continuously writes large payloads (e.g., 1 MB chunks)
- Never yields or delays
- Attempts to saturate connection capacity

**Stream B (Interactive)**:
- Sends small messages periodically (e.g., 100 bytes every 10ms)
- Time-sensitive
- Expects low latency

### Hypothesis

> If Stream A continuously writes large payloads without yielding, then Stream B sending small messages will experience:
> - Delayed delivery
> - Increased write latency
> - Possible starvation

**This should occur even though QUIC supports multiplexing.**

### Why This Is Expected

1. Stream A rapidly consumes connection flow-control credit
2. Stream B's write attempts block waiting for available credit
3. Credit only becomes available as Stream A's data is acknowledged
4. Stream B's messages queue behind Stream A's bulk data

---

## Expected Behavior

### Normal Operation (No Contention)

```
Time →
Stream A: [send] [ack] [send] [ack] [send] [ack]
Stream B: [send][ack] [send][ack] [send][ack]

Both streams progress smoothly
```

### Under Starvation

```
Time →
Stream A: [send_bulk][send_bulk][send_bulk][send_bulk]
Stream B: [blocked] [blocked] [blocked] [delayed_send]

Stream B starved by Stream A's aggressive sending
```

### Observable Metrics

| Metric | Stream A (Bulk) | Stream B (Interactive) |
|--------|-----------------|------------------------|
| Throughput | High (MB/s) | Low (KB/s) |
| Write latency | Low (buffers available) | High (frequently blocked) |
| Delivery consistency | Smooth | Bursty |
| Credit utilization | ~100% | ~0-10% |

### What We'll See

✓ **Both streams remain open** - No protocol violations  
✓ **No errors occur** - QUIC operates correctly  
✓ **Bulk stream dominates throughput** - Unfair resource allocation  
✓ **Small stream's messages arrive late** - Starvation symptoms  
✓ **Bursty delivery pattern** - Messages arrive in bursts when credit becomes available  

### Proof of Concept

This demonstrates:

> **Multiplexing ≠ Fairness**

QUIC provides the **mechanism** for concurrent streams but not the **policy** for fair scheduling.

---

### Protocol-Level Solutions

#### HTTP/3 Priorities (RFC 9218)

HTTP/3 defines priority signals:

```
Priority: urgency=3, incremental
```

- **Urgency**: 0-7 scale (0 = highest)
- **Incremental**: Boolean indicating if resource should be delivered incrementally

Servers implementing HTTP/3 priorities can schedule stream delivery accordingly.

### QUIC Implementation Extensions

Some QUIC implementations offer:
- Stream weight configuration
- Per-stream bandwidth limits
- Custom scheduling policies

Check your QUIC library's documentation (e.g., `quinn`, `quiche`, `msquic`).

---

## Real-World Implications

### When Starvation Matters

**High-impact scenarios**:

1. **Web browsers** - Latency-sensitive API calls mixed with large asset downloads
2. **Video conferencing** - Real-time audio/video alongside file transfers
3. **Gaming** - Low-latency game state updates alongside asset streaming
4. **Financial trading** - Time-critical orders mixed with market data feeds

### When Starvation Is Acceptable

**Low-impact scenarios**:

1. **Batch processing** - All streams have similar requirements
2. **Background sync** - No latency requirements
3. **Single-stream applications** - No competition

### Production Recommendations

1. **Profile your application** - Measure actual starvation occurrence
2. **Separate by latency class** - Interactive vs bulk on different connections
3. **Implement application-level fairness** - Don't rely on transport alone
4. **Monitor stream metrics** - Track per-stream latency and throughput
5. **Use HTTP/3 priorities if applicable** - Leverage existing standards

---

## Key Takeaways

### Core Concepts

1. **Multiplexing ≠ Fairness**
   - QUIC enables concurrent streams
   - QUIC does not guarantee fair resource allocation

2. **Flow Control ≠ Scheduling**
   - Flow control prevents overload
   - Scheduling determines who gets resources when

3. **Starvation Is Not a Bug**
   - It's a natural consequence of shared resources
   - Applications must implement fairness policies

### Practical Lessons

- **Don't assume fairness** - Test under contention
- **Design for starvation** - Implement mitigation strategies proactively
- **Measure in production** - Starvation may only appear under load
- **Consider connection topology** - Sometimes separate connections are the answer

### Design Principles

When building QUIC applications:

✓ **Identify latency classes** - Which streams are time-sensitive?  
✓ **Implement rate limiting** - Prevent aggressive streams from dominating  
✓ **Use priorities where available** - Leverage HTTP/3 or custom schemes  
✓ **Monitor fairness metrics** - Make starvation observable  
✓ **Test under contention** - Don't just test idle scenarios  

---

**Remember**: QUIC gives you the tools for multiplexing. You must build the fairness policies yourself.
