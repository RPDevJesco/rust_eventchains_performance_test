The EventChains Design Pattern and Rust are fundamentally at odds with one another. Do not use this code in a production context.
# EventChains Performance Comparison: Rust vs Other Languages

## Overview

This repository contains performance benchmarks for the EventChains design pattern 
implemented across multiple languages, with a focus on understanding Rust-specific 
performance characteristics.

## Key Findings

**Pattern overhead compared to traditional implementation:**

| Language | Overhead |
|----------|----------|
| C        | 10-30%   |
| C#       | 10-30%   |
| Python   | 10-30%   |
| Java     | 10-30%   |
| Ruby     | 10-30%   |
| COBOL    | 10-30%   |
| **Rust** | **5,500%-309,000%** |

## Why the Difference?

The EventChains pattern relies on multiple events accessing shared mutable state 
through a generic context (like a Dictionary/HashMap). 

**In Rust:** The borrow checker prevents holding multiple references into the 
context simultaneously, forcing full clones on every `context.get()` and 
`context.set()` operation.

For Dijkstra's algorithm with 1000 nodes:
- Each event clones ~100KB of state (DijkstraState + Graph + PriorityQueue)
- With ~1000 events, this results in ~200MB of unnecessary cloning
- This explains the 231,443% overhead
