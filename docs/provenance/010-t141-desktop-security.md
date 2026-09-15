# T141 Desktop Security and Adversarial Campaign

## Scope

T141 directly challenges the privileged desktop boundary before T142 platform qualification.

This task adds no production authority. It consolidates existing fail-closed security contracts and adds renderer-level adversarial regression coverage plus an exact-candidate workflow.

## Canonical base

- Task: `T141`
- Base merge: `4d95f04e4c8216218db737b55ba8f50649b37c43`
- Base tree: `f911bc32c1556877c7dfe2b92c09c146bf8e6e41`
- Direct Codex launch authority remains: `UNAUTHORIZED`
- Direct Claude launch authority remains: `UNAUTHORIZED`

## Privileged-boundary invariants

- Renderer host invocation remains a fixed literal command allowlist.
- No generic filesystem, shell, Git, open-path, navigation, clipboard-write, or arbitrary command dispatcher is introduced.
- Tauri capabilities remain empty and CSP remains local-only with remote connections disabled.
- Agent/user/fixture text cannot create Winds-observed or human-decided authority.
- File preview remains exact-binding, bounded, traversal-resistant, and symlink/TOCTOU fail-closed.
- Candidate/evidence/binding movement remains stale rather than silently current.

## Required campaign mapping

| Canonical adversarial class | Direct evidence |
| --- | --- |
| Forged terminal/model trusted controls and runtime marks | T089, T099, T112, T141 renderer tests |
| HTML/Markdown/script-shaped content | T141 renderer tests; React text rendering only |
| Traversal, symlink, and root/file swap | T136 Files/Changes tests |
| Malicious external links and file URLs | T095 host-safety tests; T141 URL tests |
| Clipboard-write attempts | T095 OSC52 refusal |
| Remote-origin/navigation attempts | Empty Tauri capabilities, CSP `connect-src 'none'`, T141 source scan |
| Generic invoke/command guessing | T141 fixed literal bridge allowlist |
| Oversized payload/output | T089, T095, T099, T123 and T141 oversized hostile text |
| Stale binding/result replay | T112, T136, T137 and T141 immutable-binding comparison |
| Duplicate aliases and focus races | T099 and T141 explicit ambiguity test |
| Dual-session cross-target race | T134 adversarial target race |
| Corrupt presentation metadata | T130 fail-closed presentation tests |
| Secret-shaped tracked material | T123 secret-context tests plus exact-candidate `git grep` gate |

## Exact-candidate workflow

`.github/workflows/t141-desktop-security.yml` binds checkout to the candidate SHA and runs:

- renderer format, typecheck, lint, tests, and production build on Ubuntu 24.04, macOS 15, and Windows 2025;
- T089 terminal content confinement;
- T095 host side-effect refusal;
- T134 dual-session target isolation;
- T136 traversal/symlink/binding refusal;
- T137 stale/forged evidence refusal;
- locked Tauri host build on all three desktop OS families;
- deeper T099/T112/T123/T130 authority, replay, secret, and corruption campaigns on Ubuntu;
- tracked desktop/provenance secret-shape rejection.
