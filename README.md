# EventChains Performance Comparison: Rust vs Other Languages

## Overview

This repository contains performance benchmarks for the EventChains design pattern 
implemented across multiple languages, with a focus on understanding Rust-specific 
performance characteristics.

## Key Findings

**Pattern overhead compared to traditional implementation:**

docker run --rm --privileged --cap-add=SYS_ADMIN eventchains-rust ` perf stat -d -r 10 ./dijkstra_eventchains

| Language | Overhead |
|----------|----------|
| **Rust** | 3-18%    |
