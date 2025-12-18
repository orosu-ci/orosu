## Orosu

Orosu is a CI/CD offloading tool designed to replace ad-hoc SSH/SFTP setups for build delivery.

Instead of configuring SSH keys, users, paths, and brittle scripts in every pipeline, you install Orosu once on the target machine and let CI push jobs to it.

> From Japanese 降ろす (orosu) — to unload / offload.

⸻

The Idea

CI systems are great at building but terrible at delivering:
- SSH keys spread across pipelines
- SFTP / rsync scripts copy-pasted everywhere
- fragile permissions and paths
- production servers exposed directly to CI

Orosu acts as a controlled execution point:
- CI triggers a job
- Orosu receives it
- Orosu runs predefined scripts locally

No direct SSH. No file juggling. No pipeline-specific hacks.

⸻

What Orosu Is
- A little agent running on your server
- A CLI used from CI
- A contract between CI and the target machine

⸻

What Orosu Is Not
- Not a full CI system
- Not a deployment framework
- Not an SSH wrapper

⸻

### Status

🚧 Early stage

The interface is not stable yet.
Documentation and examples will follow.

⸻

Stay tuned.