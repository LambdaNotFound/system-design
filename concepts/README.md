# Concepts

Notes on building blocks and theory: consistent hashing, replication, CAP, LSM trees, and so on. Design docs link here instead of re-explaining them.

Start one in Claude Code with `/write-concept <concept>`. Claude reads the primary sources, writes `concepts/<slug>.md` to this [spec](../.claude/skills/write-concept/note-spec.md), and adds its line to the index below. Every source in a note says whether Claude read it or cites it from memory. Annotate freely under **My notes**: Claude never edits that section, also when it revises the note.

A concept can have a crate too: `/implement-design <slug>` builds it in `src/<slug>/`, adds a `**Code:**` line under the note's title, and keeps its plan and review ledger in a folder beside the note, `concepts/<slug>/`.

## Index

- [Follower reads](follower-reads.md): a read served by a non-leader replica at a timestamp it can prove complete, trading a few seconds of staleness for a local answer
