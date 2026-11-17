# Drill record: rate-limiter

Subject: design-doc/rate-limiter/rfc.md

Q1 | 11/10/26 | hit | Sliding Window Counter | A client sends its full limit in the last second of one minute and again in the first second of the next. What does a fixed window allow, and what does the design do about it? | complete
Q2 | 11/10/26 | miss | Alternatives Considered | A token bucket in each API node adds no store call. Why does the design reject it? | each node counts alone, so the real limit is the limit times the number of nodes
Q3 | 11/12/26 | open | Fail Open | Redis does not answer within 1 ms. What does the check do with the request, and how does an operator find out? | -
