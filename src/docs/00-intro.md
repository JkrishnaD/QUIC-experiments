# QUIC Transport Playground - Introduction

This repository is not a product.
This repository is not a framework.
This repository is not a tutorial clone.

This is a **transport-layer learning lab**.

The goal of this repo is to understand **QUIC as a transport system**, not as
“the thing HTTP/3 runs on”.

---

## Why This Repo Exists

Traditional learning paths treat QUIC like this:
```text
> TCP → HTTP/1.1 → HTTP/2 → HTTP/3
```

That framing hides the real insight.

**QUIC is not an HTTP upgrade.**
It is a **new transport abstraction**.

This repo exists to:
- Understand QUIC *independently of HTTP*
- Observe real transport behavior (multiplexing, loss, migration)
- Learn by running controlled experiments
- Build intuition, not just code

---

## What We Are Trying to Learn

We focus on **transport-level questions**, such as:

- What does a QUIC connection really represent?
- Why are streams the core abstraction?
- How does QUIC avoid head-of-line blocking?
- How does packet loss affect only part of a connection?
- Why can QUIC survive IP changes?
- What does QUIC give “for free”, and what does it not?

---

## How This Repo Is Structured

- `docs/` → conceptual understanding & diagrams
- `experiments/` → minimal runnable experiments
- `src/` → shared setup / entry points

---

### Each topic follows the same learning loop:
1. Conceptual understanding (docs)
2. Expected behavior (theory)
3. Controlled experiment (code)
4. Observation & postmortem (docs update)

---

## How to Read This Repo

Read the docs **in order**.
Run experiments **only after understanding the concept**.
If something surprises you, that’s the point.

This repo is a lab notebook.
Confusion is a feature.
