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
