#!/usr/bin/env bash
# Fixture stage for this case (see evals/fixtures/place.sh), then a close and an amendment after it.
set -eu
"$(dirname "$0")/../../../../../evals/fixtures/place.sh" scoped loop-approved loop-closed impl-mapped impl-approved
d=design-doc/rate-limiter
echo "Closed: 11/12/26, approve" >> $d/impl-run.md
sed -i 's#^| \[Rate Limiter\](design-doc/rate-limiter/rfc.md) | Reviewed | 11/09/26 | — |#| [Rate Limiter](design-doc/rate-limiter/rfc.md) | Implemented | 11/09/26 | [src/rate-limiter](src/rate-limiter/) |#' README.md
new='| FR2 | A rejected response carries a `Retry-After` header with the seconds until the client may retry, and an `X-RateLimit-Limit` header with the client'"'"'s requests per minute. |'
printf '\n## Amendment 11/14/26 (after approval)\n\n| ID | Requirement |\n| --- | --- |\n%s\n\nWhy: clients asked to see their limit.\n' "$new" >> $d/requirement.md
printf '\n## Amendment A1 check 1\n\nThe FR2 row in design.md matches the amendment, and the rejection path adds the header. No objections.\n\nVerdict: approve\n' >> $d/review.md
for f in "$d/rfc.md" "$d/design.md"; do
python3 - "$f" "$new" <<'PY'
import re,sys
p,new=sys.argv[1],sys.argv[2]
s=open(p).read()
s=re.sub(r'^\| FR2 \|.*$',new.replace('\\','\\\\'),s,count=1,flags=re.M)
s=s.replace('with `Retry-After: <seconds>` (FR1, FR2)','with `Retry-After: <seconds>` and `X-RateLimit-Limit: <requests per minute>` (FR1, FR2)')
open(p,'w').write(s)
PY
done
