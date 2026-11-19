Slug: url-shortener
Brief: design-doc/url-shortener/rfc.md, design-doc/url-shortener/requirement.md
Crate: src/url-shortener
Base: 9e5bf406ced3adee2bbc9312bfc423e9d15e0c39
Started: 11/16/26 22:14
Commits: no
Plan approved: 11/16/26, 0 amendments, 0 scope lines
D1 | 11/16/26 | FR7 accepts hostless http(s) URLs (http:foo, http:///x) | accept | brief correction open: WHATWG parsing may open http:///s.example/x as http://s.example/x, bypassing FR9; amend FR7 via /new-design url-shortener later
Pass: T0 uncommitted
Pass: T1 uncommitted
Pass: R0 uncommitted
Pass: R1 uncommitted
Closed: 11/16/26, approve
Reopened: 11/19/26, A1
Commits: yes
Plan approved: 11/19/26, 1 amendments, 0 scope lines
Pass: A1.T0 272247c
Pass: A1.T1 4428bdc
Pass: A1.R0 ea6a415
Closed: 11/19/26, approve, A1
