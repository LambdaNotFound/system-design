# Sliding window counter

## What it is
A way to count one client's requests over the last N seconds from two fixed-window counters. It weights the previous window's count by how much of that window still overlaps the last N seconds, then adds the current window's count.

## Why it matters
A fixed window resets on the boundary, so a client can send its whole limit at the end of one window and again at the start of the next: twice the limit inside one window's length. An exact log of every request closes that gap and costs one stored entry per request.

Used in: no design yet.

## How it works
Keep one counter per client per window. On each request, increment the current window's counter and read the previous one:

```text
estimate = previous_count * (1 - elapsed_fraction_of_current_window) + current_count
reject when estimate > limit
```

With a limit of 100 a minute, 80 requests in the previous minute, 30 so far in this one, and 15 seconds elapsed, the estimate is `80 * 0.75 + 30 = 90`, so the request passes.

## Trade-offs and variants
The estimate assumes the previous window's requests were evenly spread. A burst late in that window is undercounted, so the limit can be exceeded by a small margin.

| Variant | State per client | Accuracy | Pick it when |
|---|---|---|---|
| Fixed window counter | 1 counter | up to twice the limit at a boundary | the limit is a rough guard |
| Sliding window counter | 2 counters | an estimate | memory matters and a small overshoot is fine |
| Sliding window log | 1 entry per request | exact | the limit is a contract |

## In real systems
- **Cloudflare:** its rate limiter uses this estimate over two counters per client, and reported a low error rate against an exact count [1].

## Check your understanding
1. A client sends its whole limit in the last second of a minute. What does the estimate say 30 seconds into the next minute, and what did the client really send in the last 60 seconds?
2. The limit drops from 100 to 10 a minute while a client holds 60 in its previous window. How long is that client rejected?
3. Two API nodes each keep their own counters. What limit does a client see, and which variant fixes it?

## My notes
- 11/12/26 the even spread assumption is the whole trick here
-   TODO compare with GCRA before the interview

## Sources
1. [How we built rate limiting capable of scaling to millions of domains](https://blog.cloudflare.com/counting-things-a-lot-of-different-things/), Cloudflare, 2017. From memory.
2. *System Design Interview*, vol. 1, ch. 4, "Design a rate limiter". From memory.
