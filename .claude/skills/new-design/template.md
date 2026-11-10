# {System Name}

**Status:** Drafting · **Started:** {MM/DD/YY} · **Code:** —

> **Summary.** <!-- Write this last: what the system does, and the one or two problems that make it hard. -->

## 1. Requirements
<!-- Scoped with /scope-problem: replace the subsections below with one line, "Signed off MM/DD/YY: [requirement.md](requirement.md)". Not scoped: fill them in. -->

### Functional
<!-- The 3–5 core capabilities. Everything else goes under "Out of scope". -->

### Non-functional
<!-- Quantified: scale (DAU, QPS, data size), latency (p99), availability, durability, and the consistency each operation needs. -->

### Out of scope

## 2. Estimates
<!-- Back-of-the-envelope: QPS (average and peak), read:write ratio, storage over N years, bandwidth, cache size. Show the math. Keep only the numbers that drive a decision. -->

## 3. Core entities and API
<!-- The main entities and how they relate. Then the API: endpoints or RPCs, key parameters, idempotency, pagination. -->

## 4. Data model and storage
<!-- Access patterns first. Then the schema, keys and indexes, the partition key, and the storage choice with its reason. -->

## 5. High-level design
<!-- Architecture diagram: diagrams/architecture.excalidraw.svg, drawn by you. Walk the main write and read paths through it. Request flows go here as Mermaid sequence diagrams. -->

## 6. Deep dives
<!-- The 2–4 hardest parts: hot paths, hot keys, consistency and ordering, fan-out. For each: problem → options → choice → why. -->

## 7. Failure modes and operations
<!-- What fails, the blast radius, how it's detected, how it recovers. Data-loss window, back-pressure, deploys and migrations, the metrics and alerts that matter. -->

## 8. Trade-offs
<!-- Decisions that could reasonably go the other way, and what would make you change them. -->

## 9. Open questions

## References
<!-- Alex Xu chapters, DDIA chapters, Hello Interview guides, papers, engineering blog posts. -->

## Review log
<!-- /review-design adds dated entries here. -->
