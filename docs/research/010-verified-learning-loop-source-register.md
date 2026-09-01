# Verified Learning Loop — Primary Source Register

Status: research provenance companion; non-authorizing.

Access date: 2026-09-01 (Asia/Riyadh).

This file binds the external research references used by `010-verified-learning-loop-roadmap.md` to stable primary or official sources. It exists only to make the roadmap's research synthesis independently reproducible. It does not amend the Winds Constitution, Spec 006, plan, tasks, runtime authority, verification authority, or implementation scope.

## Source Classification

- `PRIMARY_PAPER`: author paper / arXiv record.
- `OFFICIAL_REPOSITORY`: repository maintained by the project or authors.
- `OFFICIAL_DOCUMENTATION`: project-maintained documentation.
- `OFFICIAL_RESEARCH_PAGE`: organization/author research page for the cited work.

Where a hosted documentation page is mutable, the access date is part of the provenance record. A repository SHA is recorded only when it was independently observed; it must not be treated as a content binding for a separately hosted documentation page unless that relationship is proven.

## Research Sources

| Roadmap item | Primary / official source | Classification | Source binding | Pattern supported by the source |
| --- | --- | --- | --- | --- |
| Autoresearch | https://github.com/karpathy/autoresearch | `OFFICIAL_REPOSITORY` | reviewed revision `228791fb499afffb54b46200aca536f79142f117` | A bounded autonomous research loop modifies a limited training surface, runs fixed-budget experiments, measures the result, and keeps or discards changes. Winds borrows the experimental discipline, not a claim that one scalar metric is sufficient for software quality. |
| Hermes Agent | https://hermes-agent.nousresearch.com/docs/user-guide/features/memory/ ; https://github.com/NousResearch/hermes-agent | `OFFICIAL_DOCUMENTATION` + `OFFICIAL_REPOSITORY` | hosted docs accessed 2026-09-01; official repository `main` independently observed at `66e258b4b951fba50302a34e3af15586dbbcb796`, without claiming the hosted page is content-bound to that SHA | Hermes documents procedural skills as reusable agent-managed knowledge and documents approval gates for memory and skill writes. Winds borrows inspectable procedural knowledge and approval-aware writes, not Hermes runtime authority. |
| Reflexion | https://arxiv.org/abs/2303.11366 | `PRIMARY_PAPER` | arXiv:2303.11366 | Reflexion improves agent behavior through linguistic feedback and episodic reflective memory rather than model-weight updates. |
| Voyager | https://arxiv.org/abs/2305.16291 ; https://github.com/MineDojo/Voyager | `PRIMARY_PAPER` + `OFFICIAL_REPOSITORY` | arXiv:2305.16291; repository accessed 2026-09-01 | Voyager uses an executable skill library plus environment feedback, execution errors, and self-verification to improve behavior without fine-tuning the base model. |
| ACE | https://arxiv.org/abs/2510.04618 ; https://github.com/ace-agent/ace | `PRIMARY_PAPER` + `OFFICIAL_REPOSITORY` | arXiv:2510.04618; repository accessed 2026-09-01 | Agentic Context Engineering treats context as evolving structured playbooks and applies incremental generation, reflection, and curation rather than repeatedly replacing context with a lossy rewrite. |
| Darwin Gödel Machine | https://arxiv.org/abs/2505.22954 ; https://github.com/jennyzzt/dgm | `PRIMARY_PAPER` + `OFFICIAL_REPOSITORY` | arXiv:2505.22954; official repository `main` independently observed at `a565fd2d1dca504ef5104a7cc0f3bdc4ab9b4fd2` | DGM iteratively modifies coding agents, empirically evaluates variants, and maintains an archive that supports open-ended exploration across multiple improvement lineages. Winds borrows archive/champion-challenger ideas while explicitly deferring trusted-core self-modification. |
| PaperBench | https://arxiv.org/abs/2504.01848 ; https://openai.com/index/paperbench/ | `PRIMARY_PAPER` + `OFFICIAL_RESEARCH_PAGE` | arXiv:2504.01848; official page accessed 2026-09-01 | PaperBench decomposes research-replication work into detailed rubrics and separately evaluates the automated judge, supporting decomposed evaluation and evaluator validation. |
| RE-Bench | https://arxiv.org/abs/2411.15114 ; https://metr.org/blog/2024-11-22-evaluating-r-d-capabilities-of-llms/ | `PRIMARY_PAPER` + `OFFICIAL_RESEARCH_PAGE` | arXiv:2411.15114; METR page accessed 2026-09-01 | RE-Bench evaluates realistic ML research-engineering tasks and compares agents and human experts under explicit time budgets, supporting budget-aware evaluation. |
| SpecBench | https://arxiv.org/abs/2605.21384 ; https://github.com/WecoAI/SpecBench | `PRIMARY_PAPER` + `OFFICIAL_REPOSITORY` | arXiv:2605.21384; repository accessed 2026-09-01 | SpecBench compares visible validation tests with held-out compositional tests to measure reward hacking/generalization gaps in long-horizon coding agents. |
| Faraday / Replica | https://arxiv.org/abs/2608.13331 ; https://inherentlabs.ai/research/training-to-replicate | `PRIMARY_PAPER` + `OFFICIAL_RESEARCH_PAGE` | arXiv:2608.13331; official research page accessed 2026-09-01 | The work introduces Replica as a scalable paper-replication task space and post-trains Faraday, a higher-level AI Scientist agent that uses coding agents as tools. Winds uses this only as evidence that higher-level orchestration policy over coding-agent tools is viable, not as authority to begin learned routing. |

## Claim Boundary

These sources support only the research patterns stated above. They do not prove that those patterns are safe or optimal for Winds, and they do not authorize implementation. Before any post-Spec-006 learning program becomes formal, current source versions, licenses, applicability, evaluator design, threat model, and repository authority must be revalidated through the normal `Constitution -> Spec -> Plan -> Tasks -> Implement` process.

Research papers and external repositories are references, not source-code licenses. No source code, prompt, schema, or other protected implementation material may be copied without the existing Winds provenance and license process.
