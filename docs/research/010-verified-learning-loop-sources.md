# Verified Learning Loop Primary Source Register

**Status:** Research evidence companion. Non-normative and non-authorizing.

**Roadmap:** `docs/research/010-verified-learning-loop-roadmap.md`

**Retrieved:** 2026-09-01 (Asia/Riyadh research date)

**Purpose:** Provide stable, independently checkable primary-source identifiers for the research synthesis used by the Verified Learning Loop roadmap. This file does not authorize implementation, does not widen Spec 006/T079/T080, and does not convert external research claims into Winds product truth.

## Source classification rule

- `PRIMARY_REPOSITORY`: source maintained by the project author/owner.
- `PRIMARY_DOCUMENTATION`: official project/vendor documentation.
- `PRIMARY_PAPER`: paper/preprint published by the work's authors.
- `PRIMARY_ORG_PUBLICATION`: official publication by the organization responsible for the work.

Research papers are references, not source-code licenses. Repository code must not be copied or adapted without a separate exact-version provenance/license review under the existing Winds process.

## Primary sources

| Roadmap item | Classification | Stable primary source | What the source supports for the roadmap | Provenance / license note |
| --- | --- | --- | --- | --- |
| Karpathy Autoresearch | `PRIMARY_REPOSITORY` | https://github.com/karpathy/autoresearch | A bounded autonomous experiment loop in which an agent modifies a constrained training surface, runs under a fixed time budget, evaluates an objective metric, and keeps or discards changes based on measured outcome. | Repository reports MIT licensing; any code reuse still requires a fresh exact-pin audit. |
| Hermes Agent skills | `PRIMARY_DOCUMENTATION` | https://hermes-agent.nousresearch.com/docs/user-guide/features/skills/ | Inspectable procedural skills, agent-managed skill evolution, and optional human approval gates for skill writes. | Documentation reference only; no Hermes source code is copied by the roadmap. |
| Hermes Agent memory | `PRIMARY_DOCUMENTATION` | https://hermes-agent.nousresearch.com/docs/user-guide/features/memory/ | Bounded persistent memory and optional approval gates for memory writes. | Documentation reference only. |
| Reflexion | `PRIMARY_PAPER` | https://arxiv.org/abs/2303.11366 | Language agents can improve across trials using linguistic feedback and episodic reflective memory without model-weight updates. | Research concept reference only. |
| Voyager | `PRIMARY_PAPER` | https://arxiv.org/abs/2305.16291 | Reusable skill libraries, environment feedback, execution-error feedback, and self-verification can support lifelong agent improvement without parameter fine-tuning. | Research concept reference only. |
| ACE — Agentic Context Engineering | `PRIMARY_PAPER` | https://arxiv.org/abs/2510.04618 | Context can be evolved as a structured playbook through incremental generation/reflection/curation rather than repeatedly collapsed into a lossy summary. | Research concept reference only. |
| Darwin Gödel Machine | `PRIMARY_PAPER` | https://arxiv.org/abs/2505.22954 | Open-ended self-improvement can maintain an archive/tree of diverse agent variants and empirically evaluate changes instead of following one greedy lineage. | The roadmap explicitly does **not** authorize trusted-core self-modification. |
| PaperBench | `PRIMARY_ORG_PUBLICATION` | https://openai.com/index/paperbench/ | Complex agent work can be decomposed into hierarchical rubrics; evaluator quality itself can be benchmarked and validated. | Publication/reference only. |
| PaperBench paper | `PRIMARY_PAPER` | https://arxiv.org/abs/2504.01848 | Primary paper for the PaperBench benchmark and judge-evaluation methodology. | Research concept reference only. |
| RE-Bench | `PRIMARY_PAPER` | https://arxiv.org/abs/2411.15114 | Agent evaluation outcomes depend materially on time/resource budgets; the benchmark directly compares agents and human experts under controlled research-engineering budgets. | Research concept reference only. |
| RE-Bench archival publication | `PRIMARY_PAPER` | https://proceedings.mlr.press/v267/wijk25a.html | Archival ICML/PMLR publication of RE-Bench. | Research concept reference only. |
| SpecBench | `PRIMARY_PAPER` | https://arxiv.org/abs/2605.21384 | Visible validation-test success can diverge from held-out compositional correctness, providing a concrete reward-hacking/generalization warning for coding agents. | Research concept reference only. |
| SpecBench repository | `PRIMARY_REPOSITORY` | https://github.com/WecoAI/SpecBench | Public benchmark implementation describing visible validation suites versus held-out evaluation suites. | Repository code is not imported by Winds. |
| Faraday / Replica | `PRIMARY_PAPER` | https://arxiv.org/abs/2608.13331 | Replica provides bounded paper-replication tasks, while Faraday is a higher-level agent trained to use coding agents as tools; this supports the roadmap's claim that learned higher-level orchestration over coding agents is a viable research direction. | Research concept reference only; benchmark-specific claims must not be generalized beyond the published evaluation. |
| Faraday / Replica public explanation | `PRIMARY_ORG_PUBLICATION` | https://inherentlabs.ai/research/training-to-replicate | Official Inherent explanation of Faraday directing coding agents and Replica's task-space design. | Organization-authored result; independent replication is not implied. |

## Claim-strength constraints

The roadmap may use these sources only for the bounded patterns stated above. In particular:

1. benchmark success does not prove general product superiority;
2. organization-authored benchmark results are not independent validation;
3. a reusable skill or context technique in another agent does not prove it is safe or suitable for Winds;
4. research evidence cannot authorize Winds implementation without the normal `Constitution -> Spec -> Plan -> Tasks -> Implement` sequence;
5. no external research result can weaken Winds exact-candidate, independent-review, authority, provenance, or human-decision requirements;
6. source or license drift must be revalidated before any future implementation-derived reuse.

## Roadmap mapping

The source-supported roadmap synthesis is intentionally limited to:

```text
Autoresearch -> bounded modify/run/measure/keep-or-discard experimentation
Hermes -> inspectable procedural skills + approval-aware memory/skill writes
Reflexion / Voyager -> non-weight learning from feedback and reusable skills
ACE -> incrementally evolved structured playbooks/context
Darwin Gödel Machine -> diverse challenger/archive exploration
PaperBench -> decomposed evaluation + evaluator validation
RE-Bench -> explicit time/resource budget sensitivity
SpecBench -> visible-test vs held-out correctness / reward-hacking risk
Faraday / Replica -> higher-level policy using coding agents as tools
```

Any stronger claim requires fresh primary evidence and a new reviewed research update.