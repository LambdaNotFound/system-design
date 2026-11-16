# Software Diagram Style Guide

This guide comes from studying all 129 images of the 79 diagrams in
[terrastruct/awesome-diagrams](https://github.com/terrastruct/awesome-diagrams) (commit `c4daae8`, studied 2026-10-28).
The repo has no written commentary, so every rule here comes from looking at the images.
Each rule names repo diagrams that show it. Frequencies are approximate tallies.

Templates for every technique are in `element-templates.md`. Colors are in `color-palette.md`.

---

## 1. Pick the diagram kind first

Every good diagram in the repo answers one question with one layout. Pick the kind before you place anything.

### System architecture: "What runs where, and what talks to what?"
- **Layout:** nested boundaries (cluster > node > process). Clients and users on the left or top. Data stores on the right or bottom.
- **Must have:** a label on each boundary, one color per component role, protocol or action labels on arrows.
- **Seen in:** Kubernetes cluster architecture, Istio, Rivet, Prometheus, Lichess, Self-hosted Retool.

### Request or message flow: "What happens, in order, when X occurs?"
- **Layout:** the architecture skeleton, plus a numbered badge on every arrow of the flow.
- **Must have:** step numbers in reading order, an action on each arrow, one color per actor or flow when there are 2 or more.
- **Seen in:** Slack real-time messaging (badges 1-7, branches 5B/5C), WebSocket on Kubernetes (step text colored like its arrow), Okta (image 3).

### Data pipeline: "How does data move and change?"
- **Layout:** one direction, usually left to right. Stages as boxes, stores as cylinders.
- **Must have:** the payload ("Chunks", "Embeddings") or the tool ("Debezium CDC", "Hudi + Spark") on every arrow.
- **Seen in:** Notion's data lake, Contextual Retrieval, Lambda architecture, Basic Airflow deployment.

### Sequence: "In what order do a few actors exchange messages?"
- **Layout:** one lifeline per actor, time flows down. Headers at the top.
- **Must have:** arrows colored by the sending actor, a color legend, a label on every message.
- **Optional:** a faint dashed bar for a sub-flow you skip ("Authenticate user").
- **Seen in:** WorkOS SSO, Google SSO.

### Layered stack: "What is built on what?"
- **Layout:** horizontal bands, top to bottom. No arrows needed; adjacency means "uses".
- **Must have:** one color per layer, a band title with a small subtitle (for example the language).
- **Seen in:** Flutter (4 views, one template), ExecuTorch, Faiss.

### State machine or workflow: "What states exist, and how do they change?"
- **Layout:** the main path in one row. Side states in a row below.
- **Must have:** main-path states filled, side states outlined only. Each transition labeled with its condition ("Return true", "Catch #1").
- **Seen in:** Adyen payments lifecycle, AWS Step Functions, Strapi.

### Comparison or before/after: "What is the difference?"
- **Layout:** draw the same canvas twice and change only the difference. Or use a grid of small panels with one shared template.
- **Must have:** the change highlighted in one color, everything else neutral. Paired titles ("Without VNodes" / "With VNodes").
- **Seen in:** Pinecone Global API (identical except one new node), Contextual Retrieval (new parts recolored blue), Okta (4-step reveal), ScyllaDB, The Big LLM Architecture Comparison (2x3 panels), ActiveMQ (mirrored columns).

### Concept or algorithm: "How does an idea work?"
- **Layout:** a structure that mirrors the idea. Use one of the layout patterns in section 12.
- **Must have:** concrete example values, not just names.
- **Seen in:** HNSW (3 panels over time), The Illustrated Transformer (3 zoom levels), Redux (a loop), RocksDB.

---

## 2. Group by real boundaries

About 6 in 10 diagrams group components inside boundaries. A boundary is a real thing: a cluster, region, VPC, node, process, team, trust zone, or phase.

| Boundary type | Style | Seen in |
|---|---|---|
| Network, trust, or deployment zone | Dashed rounded rectangle, very light fill or none | Slack (Edge Region, Main Region), Jenkins, Knative, Istio |
| Product or system boundary | Solid rounded rectangle, light fill | Kubernetes (Cluster), Jitsi, dbt |
| Logical layer, plane, or phase | Tinted band with no border | Istio (data plane, control plane), Contextual Retrieval, Ragflow |
| Flat tiers | Dashed vertical divider lines with a title at the top of each lane | AWS Serverless, Brex |

Rules:
- Put the label at the **top-left corner**, inside the boundary. Use 16 px text in the boundary-label color.
- Keep 30-40 px of padding inside a boundary. Start the first child at least 44 px below the boundary's top edge, so the label has room.
- Nest at most **3 levels**. Give each level a different style, for example a dashed zone, then a solid box, then a band. Four similar dashed levels were hard to tell apart (AWS ElastiCache).
- One boundary per real thing. Do not draw a box just to make the layout look tidy.

---

## 3. Same kind, same look

Readers learn the vocabulary once, then read the whole diagram.

| Thing | Shape | Seen in |
|---|---|---|
| Service, process, component | Rounded rectangle | almost all |
| Database, cache, store | Cylinder | Rivet, Airflow, Slack (Vitess), Lambda architecture |
| Human user or actor | Person icon | Airflow, LocalAI, Scalable WebSocket |
| External system, SaaS, the Internet | Gray box with hatch fill, or a cloud | WebSocket on Kubernetes (hatched Kubernetes API), Kubernetes (cloud provider API), Lichess |
| Proxy, load balancer, router | Hexagon or diamond | Istio (hexagon proxy), Sentry, WebSocket on Kubernetes (diamond) |
| Decision | Diamond | 8 RAG architectures, DuckDB flowchart |
| Input or output data | Parallelogram | 8 RAG architectures |
| Many instances of one thing | 2 offset copies behind the box, or "×N" | Slack, Envoy, NGINX, Lichess ("stockfish ×18") |

Rules:
- Use **2-5 shape kinds** per diagram. Pinecone Global API used 2 shapes and no color, and stayed clear.
- Same kind of component means the same shape, size, fill, and stroke.
- Use fill vs outline for a second meaning within one shape, for example a filled ellipse for global middleware and an outlined one for per-route middleware (Strapi).

---

## 4. Color encodes one thing

About 6 in 10 diagrams use color as a code. The good ones pick **one** dimension and stick to it.

| Dimension | Example |
|---|---|
| Component role | Kubernetes (dark blue core process, light blue cloud-facing component) |
| Owner or trust boundary | Auth0 (one fill per company), Acrolinx |
| Layer | Flutter, Faiss, Transformer (color per layer type, so repeats read instantly) |
| Flow or actor | WebSocket on Kubernetes (Client A blue, Client B orange), Nomad, Scalable WebSocket (only the actors are colored) |
| Traffic type, on arrows | Jenkins (SSH red, HTTP blue, SMTP purple), RabbitMQ, NGINX |

Rules:
- Use 2-5 category colors. Keep everything else neutral gray.
- If all boxes have the same role, put the color on the **arrows** instead (RabbitMQ).
- When an arrow has a color, give its label and its step badge the same color. This links each step to its description without a legend (WebSocket on Kubernetes, Jenkins).
- Pair color with a second cue when the meaning is critical: a check or cross icon, a label, or a line style (ArtPrompt jailbreak).
- In a before/after diagram, color only the changed parts (Contextual Retrieval).
- Color that decorates instead of encoding was a weakness (Advanced RAG, part 2).

---

## 5. Label every arrow

About half of all diagrams label their arrows, and nearly every strong flow diagram does. An unlabeled arrow makes the reader guess.

| Label type | Example | Seen in |
|---|---|---|
| Action, as a verb | "author", "read", "install", "pull metrics" | Airflow, Prometheus, Jitsi |
| Payload, as a noun | "question", "Chunks", "Top K chunks" | 8 RAG architectures, Contextual Retrieval |
| Protocol and port | "HTTP (8080)", "gRPC", "Websocket" | Jenkins, Steampipe, WebSocket on Kubernetes |
| Tool that does the hop | "Debezium CDC", "Hudi + Spark" | Notion's data lake |
| Literal route or call | `302 /authorize?client_id=`, `loadContent` | Auth0, Sentry, Docusaurus |
| Condition | "Return true", "Over limit", "HIT" / "MISS" | Strapi, DuckDB flowchart, Vercel |

Rules:
- Keep labels to 1-4 words. Put literal strings in the monospace font.
- Bind the label to the arrow. Excalidraw then centers it on the arrow and cuts a gap in the line.
- Make the arrow at least 60 px longer than its label, or the gap swallows the line.
- Two-way arrows are fine for request and response. Use two one-way arrows when the two directions carry different things.
- Two arrows between the same two boxes in the same direction: merge them into one arrow with one label ("GET original, PUT 3 sizes"). Otherwise their centered labels land on top of each other.

---

## 6. Use line style for one more meaning

About 1 in 4 diagrams uses line style as a code. Decide the meaning once and apply it to every arrow.

| Style | Common meaning | Seen in |
|---|---|---|
| Solid | Request, call, or data flow | almost all |
| Dashed | Async event, control or config, remote call, "can be disabled" | Istio (config), Kubernetes (remote), Jenkins (can be disabled), Knative (watch) |
| Dotted | Polling, optional input, or a secondary link | Airflow (DB polling), Flux, How to train LLMs |
| Thick | The main or critical path | Lichess, K8sGPT, Flux |
| Thick hollow two-way | A long-lived connection, such as a WebSocket | Slack |
| Faint dashed two-way bar | A skipped sub-flow in a sequence | WorkOS |

Rules:
- Use at most 2 meanings beyond plain solid. Reusing dashed for 2 meanings was a weakness (Kafka Streams).
- If line style means anything other than the table above, define it in a legend (Jenkins, Knative).
- Use right-angle (orthogonal) connectors in architecture diagrams; about 3 in 4 diagrams in one batch did. Use curves for feedback, return, and skip paths (Transformer residuals, Airflow "operate").

---

## 7. Number the steps of a flow

Step numbers appear in about 1 in 7 diagrams overall. They appear in most request-flow and sequence diagrams, where they matter most.

Rules:
- Put a small numbered badge on or next to each arrow, in reading order. Start at 1; use 0 for setup that happens before the flow (WebSocket on Kubernetes).
- Use letters for parallel branches: 5B and 5C go to two regions at once (Slack).
- Reuse a number for the same step happening in parallel lanes (Scalable WebSocket).
- Center each badge on its arrow, 24 px from the arrow's start. Excalidraw puts the label at the arrow's midpoint, so the arrow must be at least its label width + 100 px long to hold both.
- In a pipeline where each step is a box, number the boxes instead: center the badge on each box's top-left corner (Firebase numbers its lanes; Advanced RAG numbers its stages).
- Alternative: write the number into the label ("1. GET /login"), as in sequence diagrams (Google SSO).

---

## 8. Show many with one

About 1 in 4 diagrams shows multiplicity with a shortcut instead of drawing every copy.

| Technique | Seen in |
|---|---|
| 2 offset copies stacked behind the box | Slack, Envoy, NGINX, Istio, Prometheus |
| "×N" text next to or inside the box | Lichess ("stockfish ×18"), YOLOv5 ("×3"), Transformer ("Nx") |
| "Instance 1", "Instance 2", then "…" or a dotted line, then "Instance N" | Scaling DuckDB, WebSocket on Kubernetes |
| Small repeated squares inside a box | Redux ("R" reducers), Kubernetes (pods) |

---

## 9. Push the outside to the edge

- Draw external systems and anything out of scope in gray, with a hatch fill or a dashed outline (WebSocket on Kubernetes).
- Place them at the canvas edge, outside your main boundary (Kubernetes cloud provider API, Jenkins Internet zone).
- Label them with what they are to you: "Stripe API", "Customer IdP".

---

## 10. Be concrete

The strongest diagrams show real details next to the boxes.

- Real names: binary names under the role label ("kube-scheduler" under "scheduler", Kubernetes), tech choices in parentheses ("Load balancer (e.g. nginx)", Scalable WebSocket).
- Real strings: routes, ports, topics, and method names on arrows (Auth0, Sentry, Jenkins, Docusaurus).
- Real data: a JSON payload inside the node that sends it (Redux), a sample query in a code block (Netlify Connect), a checklist inside a node of what that step validates (Vercel).
- A caption under a box saying **why** it exists, not just what it is (Notion's data lake).
- Failure modes attached right under the node they affect, in the error color (Galileo).

---

## 11. Text

| Role | Size (px) | Color | Font |
|---|---|---|---|
| Title | 28 | main text | sans (`fontFamily: 9`) |
| Boundary or lane label | 16 | boundary-label color | sans |
| Component name | 16 | main text | sans |
| Edge label | 14 | edge-label color, or the arrow's color | sans; mono for literals |
| Caption, subtitle, note | 13 | caption color | sans |
| Literal: command, route, code, JSON | 13-14 | main text, or code colors on a dark block | mono (`fontFamily: 3`) |

Rules:
- Use sans-serif for names and labels; the repo diagrams almost all do.
- Switch to monospace only for literal text. The font switch itself says "this is real code" (Auth0, Docker, Redux, Faiss).
- Excalidraw has no bold or italic. Build hierarchy with size and color only.
- Use a two-line pattern for detail: the name inside the box, and a 13 px caption below it for the technology, binary name, or count.
- Size each box to fit its text. Clipped labels were a weakness (Self-hosted Retool).
- A short caption under the whole diagram names it ("Journey of a message", Slack).

---

## 12. Layout

- Pick one main direction: left to right for pipelines and request flows, top to bottom for stacks and hierarchies.
- Align to a 20 px grid. Same-kind components share a row or column.
- Keep at least 100 px between connected components, and enough that each arrow is 60 px longer than its label.
- When 3 or more arrows leave one box, give each its own attach point, at least 40 px apart. Make the box taller or use a second side if needed. For one-to-many, draw one labeled trunk that splits into branches (8 RAG architectures, "lookup" to 3 stores).
- Keep captions and boundary labels out of arrow paths. Put a caption on a side of its box that no arrow uses.
- Avoid crossing lines. Crossing many-to-many edges was the most common weakness in the repo (Sentry, Rivet, RabbitMQ, Docusaurus). Fixes: reorder components, route around with right angles, color each edge by its source (Self-hosted Retool), or split the diagram.
- Keep the long side at most about 2000 px. Bigger diagrams get shrunk when viewed. Split them into a series instead (section 13).

### Layout patterns for concepts
| If the concept... | Use | Seen in |
|---|---|---|
| Spawns many outputs | Fan-out from one node | Steampipe |
| Combines many inputs | Convergence into one node | Netlify Connect |
| Loops | A cycle with an arrow back to the start | Redux, Airflow ("operate") |
| Is built in layers | A stack of bands | Flutter, MongoDB |
| Is a ring or hash space | A circle of nodes | ScyllaDB, Weaviate |
| Changes over time | Panels left to right | HNSW |
| Has a hierarchy | A tree, top to bottom | Pinecone Global API |
| Compares options | Side by side, mirrored | ActiveMQ |

---

## 13. Series: reuse one canvas

- **Progressive zoom:** draw the same scene 2-3 times at more detail, with the same colors (The Illustrated Transformer).
- **Progressive reveal:** keep the canvas fixed and change one layer per image (Okta).
- **Variants:** one template for a family of diagrams (Flutter native, web, and detailed; Slack's 3 flows; 8 RAG architectures).
- **Before/after:** identical except the change (Pinecone Global API).

---

## 14. Legends and titles

- Add a legend only when color or line style means something the labels do not say. Only about 1 in 7 diagrams needed one.
- When you need one, define everything at once: colors, line styles, and arrowheads (Jenkins defines all 3).
- Put it in a corner box, top-left or bottom-right. Use swatches for colors and short line samples for line styles.
- Cover every color and line style you use. A legend that leaves some out misleads (Galileo).

---

## 15. Mistakes seen in the repo

| Mistake | Seen in | Instead |
|---|---|---|
| Crossing, unlabeled many-to-many arrows | Sentry, Rivet, RabbitMQ | Re-layout, label, or split |
| Text clipped by its box | Self-hosted Retool | Size boxes to the text |
| Too many similar nested boundaries | AWS ElastiCache | At most 3 levels, each styled differently |
| Color or line code with no legend | LocalAI, OpenChat, Terraform Enterprise | Add a legend |
| Decorative color | Advanced RAG (part 2) | Color one dimension |
| Text too small to read | Flutter (part 3), YOLOv5 | Captions at least 13 px |
| Too many techniques in one image | DuckDB + dbt + Airflow pipeline | Pick the 2-3 that answer the question |
| Meaning carried only by vendor logos | AWS Serverless, Notion's data lake | Text labels; logos are not available here |

---

## 16. What Excalidraw cannot do, and the substitute

| Repo technique | Substitute |
|---|---|
| Vendor logos and icons | A text label in the role color, plus the shape vocabulary in section 3 |
| Bold or italic text | Size and color |
| Gradients and 3D | Flat fills; a band per layer |
| Hexagons, parallelograms, clouds | Closed `line` shapes (templates provided) |
| Dash-dot lines | Dotted lines, or "…" text |
