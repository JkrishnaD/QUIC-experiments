# QUIC Streams & Multiplexing

Streams are the core abstraction of QUIC and the primary reason the protocol was designed. This section explores what streams are, why they exist, and how they differ fundamentally from TCP's single byte stream model.

---

## What a Stream Is (and Is Not)

### A QUIC Stream IS:

- **An ordered byte stream** - Data arrives in the order it was sent within that stream
- **Independently flow-controlled** - Each stream has its own flow control window
- **Independently retransmitted** - Lost packets affect only their own stream
- **Scoped to a single connection** - Streams cannot exist outside their parent connection
- **Lightweight** - Creating streams has minimal overhead
- **Bidirectional or unidirectional** - Can be configured for one-way or two-way communication

### A Stream is NOT:

- **A socket** - Sockets are OS-level primitives; streams are QUIC-level abstractions
- **A connection** - Multiple streams share one connection
- **A thread** - Streams are multiplexed, not parallel execution units
- **A TCP stream** - Fundamentally different in isolation and multiplexing behavior

---

## Why Streams Exist

### The TCP Problem: Head-of-Line Blocking

TCP provides exactly **one ordered byte stream per connection**.

**The issue**: If a single packet is lost, **all subsequent data is blocked** until that packet is retransmitted and received, even if the blocked data belongs to logically independent messages.

This is known as **head-of-line (HOL) blocking** and it affects performance dramatically, especially on high-latency or lossy networks.

### The QUIC Solution

QUIC solves this by moving **multiplexing into the transport layer itself**:

- Each stream is isolated from others
- Packet loss in stream A does not block stream B
- Applications can use multiple streams for independent resources
- Retransmissions are scoped to the affected stream only

---

## Transport-Native Multiplexing

### How QUIC Multiplexing Works

In QUIC:

- **Multiple streams coexist** within a single connection
- **Packet loss affects only the involved stream** - Other streams continue uninterrupted
- **Each stream maintains its own state** - Sequence numbers, flow control, delivery guarantees
- **Stream IDs identify data** - Frames carry stream ID, enabling demultiplexing at the receiver

### Visual Comparison

**TCP (Single Stream)**:
```
Connection: [A₁][A₂][B₁][A₃][B₂]
                   ↑
              Lost packet
                   ↓
         [A₁][ ? ][BLOCKED][BLOCKED][BLOCKED]
         
All data blocked until A₂ is retransmitted
```

**QUIC (Multiple Streams)**:
```
Connection:
  Stream A: [A₁][A₂][A₃]
                  ↑
            Lost packet
                  ↓
  Stream A: [A₁][ ? ][A₃]  ← Only Stream A affected
  Stream B: [B₁][B₂]       ← Stream B continues normally
```

## QUIC vs HTTP/2 Over TCP

A common misconception is that HTTP/2 already solved this problem. It didn't.

### HTTP/2 Over TCP

- **Application-layer multiplexing** over a single TCP connection
- TCP still has HOL blocking at the transport layer
- Lost TCP packet blocks **all HTTP/2 streams** until retransmitted
- Multiplexing is an illusion above a single blocking stream

### QUIC with HTTP/3

- **Transport-layer multiplexing** with true stream isolation
- Lost QUIC packet blocks **only the affected stream**
- Other streams continue independently
- True multiplexing at the protocol level

**Result**: QUIC with HTTP/3 can outperform HTTP/2 over TCP, especially on lossy networks (mobile, Wi-Fi, long-distance connections).

---

## Stream Properties in Detail

### 1. Ordered Delivery Within Stream

Data sent on stream A arrives in order for stream A, but stream A and stream B have independent ordering.

```rust
// Stream A sends: [1][2][3]
// Stream B sends: [X][Y][Z]

// Receiver might see:
// Stream A: [1][2][3] ✓ ordered
// Stream B: [X][Y][Z] ✓ ordered
// But interleaving at packet level is arbitrary
```

### 2. Independent Flow Control

Each stream has its own credit-based flow control:

```
Stream A: 100 KB remaining credit
Stream B: 200 KB remaining credit
Connection: 500 KB remaining credit (shared)
```

- Stream-level credit prevents one stream from hogging bandwidth
- Connection-level credit limits total connection usage
- Sender cannot exceed either limit

### 3. Independent Retransmission

Lost data is retransmitted per-stream:

```
Stream A: Lost packet 5 → Retransmit only packet 5 of Stream A
Stream B: Continues delivering packets 1, 2, 3 without waiting
```

### 4. Lightweight Creation

- Creating a new stream is cheap (just a stream ID)
- No handshake required
- No additional round trips
- Applications can create streams freely

---

## Stream Types

### Bidirectional Streams

- **Both peers can send and receive**
- Initiated by either client or server
- Each direction can be closed independently (half-close)
- Use case: Request-response patterns

```rust
let (mut send, mut recv) = connection.open_bi().await?;
send.write_all(b"request").await?;
send.finish().await?;
let response = recv.read_to_end(1024).await?;
```

### Unidirectional Streams

- **Only the initiator can send**
- Receiver can only read
- Lower overhead than bidirectional
- Use case: Push notifications, streaming data

```rust
let mut send = connection.open_uni().await?;
send.write_all(b"notification").await?;
send.finish().await?;
```

---

## Stream Lifecycle

### Creation

- Client-initiated: Even-numbered stream IDs
- Server-initiated: Odd-numbered stream IDs
- Bidirectional vs unidirectional determined by ID range

### Active

- Data is sent and received
- Flow control enforced
- Retransmissions happen as needed

### Half-Closed

- One direction finished (via `finish()`)
- Other direction still active
- Common in request-response: send request, finish send, wait for response

### Fully Closed

- Both directions finished
- Stream resources can be reclaimed
- Stream ID cannot be reused

---

## Observations from Experiment

- Multiple bidirectional streams were opened concurrently over a single QUIC connection.
- Streams were accepted by the server in a non-deterministic order.
- Data chunks from different streams arrived interleaved in time.
- Each stream maintained its own independent byte offset sequence.
- Stream completion times varied based on per-stream behavior.
- Slower streams did not block faster streams from making progress.

### Ordering Guarantees

Even when multiple streams transmit data with identical delays,  
the order in which chunks arrive across streams is not deterministic.

This is expected behavior.

QUIC guarantees:
- reliable, ordered delivery **within a single stream**

QUIC does **not** guarantee:
- ordering across streams
- fairness between streams
- deterministic arrival order

### Why This Happens

Cross-stream ordering can be affected by:
- async runtime wake-up order
- QUIC stream scheduling
- packet dispatch and buffering

---

### Conclusion

This experiment confirms that QUIC provides **transport-native multiplexing**  
and eliminates head-of-line blocking present in TCP-based protocols.

Understanding stream independence is essential to understanding  
why QUIC exists and when it should be used.
