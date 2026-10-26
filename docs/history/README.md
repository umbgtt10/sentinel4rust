# docs/history/

Point-in-time documents -- decisions that were replaced, plans that ran their course. Not living documentation: for
the current state, see `docs/ARCHITECTURE.md`, `docs/ADRs/` and the README.

Every file carries a tag in its name:

| tag | meaning |
|---|---|
| `[SUPERSEDED]` | Was decided, and perhaps built, and a later decision replaced it. |
| `[IMPLEMENTED]` | A plan that shipped as described. |
| `[REJECTED]` | Considered and not adopted. |
| `[FINDINGS]` | An investigation or census, not a decision. |

A tag describes the document's status when it was moved here, not a live guarantee: check a claim against the source
before citing it.

| file | what it was |
|---|---|
| [`[SUPERSEDED] DESIGN.md`](%5BSUPERSEDED%5D%20DESIGN.md) | The design the implementation was built from, revision 3.1, moved here unchanged. Its decisions are now `docs/ADRs/`, its structure `docs/ARCHITECTURE.md`; `docs/DESIGN.md` says where each section went. |
| [`[IMPLEMENTED] CORE_CLI_SPLIT.md`](%5BIMPLEMENTED%5D%20CORE_CLI_SPLIT.md) | The plan that split a technology-free `no_std` core from a cli crate, ten slices, as it landed; `docs/CORE_CLI_SPLIT.md` points here. |
