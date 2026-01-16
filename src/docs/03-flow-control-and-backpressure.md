# QUIC Flow Control & Backpressure

After understanding connections and streams, the next critical question is:

> Who is allowed to send how much data, and what happens when the receiver is slow?

This is the problem of **flow control**.

---

## What Flow Control Is

Flow control is a **receiver-driven mechanism** that limits how much data
a sender may have in flight.

Its purpose is to:

- prevent receiver memory exhaustion
- allow slow receivers to apply backpressure
- maintain system stability

Flow control is about **capacity**, not speed.

---

## What Flow Control Is NOT

Flow control is NOT:

- congestion control (network capacity)
- packet loss handling
- bandwidth estimation

Congestion control reacts to the **network**.  
Flow control reacts to the **receiver**.

QUIC implements both, but they are distinct.

---

## Two Levels of Flow Control in QUIC

QUIC applies flow control at **two levels**:

### 1. Stream-Level Flow Control

- Limits how much data can be sent on a single stream
- Prevents one stream from overwhelming the receiver

### 2. Connection-Level Flow Control

- Limits total data across all streams
- Prevents aggregate memory exhaustion

Both limits must allow progress for data to flow.

---

## Backpressure

When a receiver:

- stops reading data
- or reads data slowly

It does NOT increase its flow control window.

As a result:

- the sender eventually exhausts its window
- sending stalls
- progress resumes only after the receiver reads more data

This propagation of slowdown is called **backpressure**.

---

## Why Flow Control Matters

Without flow control:

- fast senders can crash slow receivers
- memory usage becomes unbounded
- fairness is impossible

Flow control is essential for:

- multiplexed protocols
- long-lived connections
- reliable streaming systems

---

## Experiment

### Goal - To see sender-side staling caused by slow reciever.

- sender starts normally
- reciever reads very slowly
- sender eventually stalls
- progress resumes only after the receiver reads more data

### Client behaviour

- open one stream
- continuously write data to the stream
- log timestamps before each write
- detect when write stalls

This shows sender backpressure

### Server behaviour

- accept the stream
- read one chunk at a time
- sleep for a short duration after each read
- repeat

This stimulates slow receiver

--- 

## Observations from Experiment

- When the receiver reads slowly, the sender’s `write_all().await` begins to block.
- The delay observed on the sender correlates directly with the receiver’s read interval.
- Flow control operates on **bytes in flight**, not on the number of write calls.
- Small writes (e.g. 1 byte) consume flow-control credit slowly and delay occurs infrequently.
- Larger writes (e.g. 1 KB) consume flow-control credit quickly and cause frequent stalls.
- Data received by the server is not aligned with sender write boundaries.
- The server may receive large contiguous byte chunks even if the client writes smaller chunks.

---
## Output

<img width="500" height="590" alt="image" src="https://github.com/user-attachments/assets/f1263918-79ac-4670-9149-5557bfc4e7ba" /> <img src="https://github.com/user-attachments/assets/9c77f262-2cec-40de-8213-956e46fa37d9" width="500" height="600"/> 

--- 

## Interpretation of the Output

 - The uneven arrival size and timing of received data confirms that QUIC streams are
byte streams rather than message-based channels.

 - Write boundaries on the sender side are not preserved on the receiver side.
Instead, QUIC delivers contiguous byte ranges based on availability, buffering,
and flow-control state.

 - Backpressure manifests as increased latency in `write_all().await`,
not as explicit errors or dropped data.

--- 

## Conclusion

This experiment demonstrates that QUIC implements receiver-driven flow control
using byte-based credit windows.

Backpressure is an expected and essential behavior that emerges when:
- the receiver reads slowly
- and available flow-control credit is limited

Understanding flow control is critical for designing:
- streaming protocols
- multiplexed transports
- fair and stable long-lived connections

Flow control explains *when* progress stops.
It does not explain *who* should progress next.
That problem is addressed by scheduling and prioritization.
