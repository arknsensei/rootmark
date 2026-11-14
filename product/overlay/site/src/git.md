# How RootMark differs from git

*"My notes are already in git. Why another tool?"* — git and RootMark answer different questions, and they work side by side: RootMark never writes into your Markdown files, so a vault can stay a git repository.

Every claim on this page links to the record that measured it, in the development repository at the tag of this release (`v0.4.0`), and states the condition it was measured under. Where a claim holds only under a condition, the condition is part of the claim.

## What git gives you, and RootMark does not

- **Full history of every byte, branches, merges, and remote sync.** RootMark has no branches and does not sync between machines; it is one process beside one folder on one machine. [^nosync]
- **A backup.** RootMark's store (`<vault>/.rootmark/`) is derived from your files and can be rebuilt from them; it is not a copy of them. [^derived]

If what you need is "what did this file look like last week, and who committed it", git is enough.

## What RootMark adds

| Question | git | RootMark | Condition |
|---|---|---|---|
| Which earlier text did this paragraph come from? | Line blame within one file; a moved, split or merged paragraph starts over | Every **block** has a lineage that follows it across edits, moves between files, splits and merges [^lineage] | lineage is inferred from content (spec §5, matcher v3); measured on the conformance fixtures and the prototype's synthetic and real histories |
| Who wrote this block — a person or an agent, and which one? | The committer of the commit, per file; any name can be set | Per block: author kind (human, agent, rule), the agent's name, the channel (MCP, HTTP, file watcher) [^prov] | the agent's name is a declaration, not an authenticated identity [^rules] |
| Is this approved knowledge or a suggestion? | No notion of approval inside a file | Agents' writes through MCP are `proposal`s; they are **not found by search** until a person promotes them [^layers] | the default `human_only` promotion; or rules a person wrote in advance for given paths and agent names [^rules] |
| Can an agent overwrite approved text? | Anyone with write access can | Not through MCP: under the default `append_only` edit policy an agent's change of a `canonical` block is refused and nothing is written [^append] | the MCP channel; see the next row for direct file writes |
| What if the agent writes the file directly (not through MCP)? | — | With the **default** policy the watcher records the change as a person's edit (`canonical`). It is kept out of `canonical` only with `[attestation] unattested = "unverified"` **and** an attesting client for people (the Obsidian plugin or `rootmark attest`) [^attest] | measured: with that option, renames can leave some of a person's own content `unverified` (151 of 2 228 blocks on the official seed); a process that can read the token or drive the person's client counts as the person [^threat] |
| Which notes did this answer rest on? | — | `search` and `read_block` record the blocks an answer used; `trace` lists them [^trace] | answers given through RootMark's tools |
| What did the knowledge base say yesterday? | Check out an older commit (whole files) | Name a checkpoint; search and blame **at** that checkpoint, block by block, without changing the files [^checkpoint] | |
| Will two machines agree? | Same bytes, same hashes | The same files give the same event log hash and the same index bytes on every machine — the embedder's vectors are byte-equal on x86_64 and aarch64 Linux and macOS arm64 [^determinism] | the pinned default model; a release's own binary |

## In one sentence

git records **which bytes changed in which file, and who committed them**; RootMark records **which block each block came from, who (person, agent, rule) wrote it through which channel, whether it is approved, and which blocks an answer rested on** — under the conditions above.

## Records

[^nosync]: The project's scope: no sync yet; a sync round is planned after the external pilot. [docs/ROADMAP.md §2 (P2-C)](https://github.com/arknsensei/rootmark/blob/v0.4.0/docs/ROADMAP.md) (Turkish).

[^derived]: The specification's object store and event log, §4, and the rule that nothing is written into Markdown: [spec/SPEC.md](https://github.com/arknsensei/rootmark/blob/v0.4.0/spec/SPEC.md) §4.

[^lineage]: [Round P1-A, final report](https://github.com/arknsensei/rootmark/blob/v0.4.0/reports/rounds/P1-A/FINAL.md): conformance Level 1, C1 17/17 (A-1); the prototype rounds v1–v3 (synthetic edits, splits and merges, and a real history of 56 000 blocks): the project's [NORTH_STAR.md](https://github.com/arknsensei/rootmark/blob/v0.4.0/NORTH_STAR.md), "proven" (Turkish).

[^prov]: [Round P1-B, final report](https://github.com/arknsensei/rootmark/blob/v0.4.0/reports/rounds/P1-B/FINAL.md) (provenance and layers, spec §4.3, §7) and [round P1-D.1](https://github.com/arknsensei/rootmark/blob/v0.4.0/reports/rounds/P1-D.1/FINAL.md) (change and delete events carry their actor and source).

[^layers]: [Round P1-B](https://github.com/arknsensei/rootmark/blob/v0.4.0/reports/rounds/P1-B/FINAL.md) and [P1-B.1](https://github.com/arknsensei/rootmark/blob/v0.4.0/reports/rounds/P1-B.1/FINAL.md) (spec §7, conformance C2 10/10); end to end in the product scene, steps 3–7: [round P1-E](https://github.com/arknsensei/rootmark/blob/v0.4.0/reports/rounds/P1-E/FINAL.md) (11/11).

[^rules]: [Round P2-P, final report](https://github.com/arknsensei/rootmark/blob/v0.4.0/reports/rounds/P2-P/FINAL.md): promotion rules (`[[promotion.rules]]`); an unattested file write is never promoted by a rule; the agent's name is not authenticated.

[^append]: [Round P1-D, final report](https://github.com/arknsensei/rootmark/blob/v0.4.0/reports/rounds/P1-D/FINAL.md): `append_only` is the default; the agent's attempt in step 10 of the product scene is refused and nothing is written ([P1-E](https://github.com/arknsensei/rootmark/blob/v0.4.0/reports/rounds/P1-E/FINAL.md)).

[^attest]: [Round P2-0.3, final report](https://github.com/arknsensei/rootmark/blob/v0.4.0/reports/rounds/P2-0.3/FINAL.md): with `unverified` and the attesting client, no unattested direct write became `canonical` (0 of 525 blocks in 200 scenarios; 0 of 210 in 120 more); the rename threshold (≤ 0.01 wrongly `unverified`) was missed at 0.0678 (151/2228), which is why the default stays `canonical` (decision K-P2-03.1, [docs/DECISIONS.md](https://github.com/arknsensei/rootmark/blob/v0.4.0/docs/DECISIONS.md), Turkish).

[^threat]: [Threat model](threat-model.md): the token and the human client are the human channel.

[^trace]: [Round P1-B](https://github.com/arknsensei/rootmark/blob/v0.4.0/reports/rounds/P1-B/FINAL.md) (consumption traces, spec §8); step 8 of the product scene ([P1-E](https://github.com/arknsensei/rootmark/blob/v0.4.0/reports/rounds/P1-E/FINAL.md)).

[^checkpoint]: [Round P1-B](https://github.com/arknsensei/rootmark/blob/v0.4.0/reports/rounds/P1-B/FINAL.md) (checkpoints, spec §8) and [P1-C](https://github.com/arknsensei/rootmark/blob/v0.4.0/reports/rounds/P1-C/FINAL.md) (search at a checkpoint); step 9 of the product scene.

[^determinism]: [Round P2-V.1, final report](https://github.com/arknsensei/rootmark/blob/v0.4.0/reports/rounds/P2-V.1/FINAL.md) (vector bytes equal on three platforms) and [P1-A](https://github.com/arknsensei/rootmark/blob/v0.4.0/reports/rounds/P1-A/FINAL.md) (event-log hash equal across independent runs).
