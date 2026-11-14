# Security

## Reporting a vulnerability

Please **do not** open a public issue for a security problem. Use GitHub's
private vulnerability reporting: the repository's **Security** tab → **Report a
vulnerability**.

Include what you can:

* What the issue is and where (file, command, or endpoint).
* How to reproduce it, ideally the smallest sequence that shows it.
* What an attacker could do with it.
* Your environment (OS, version, how you installed eQuran).

You can expect an acknowledgement within a few days. This is a volunteer project,
so there is no bug bounty.

## Scope

eQuran is a local desktop and command-line application. It has no server, no
account and no telemetry. The things worth reporting are the ones that would
break that model:

* A path where content, a search query, or a synthesized string causes data to be
  sent somewhere other than the documented sources.
* Anything that lets cached files, settings, or the search index be read or
  modified by another local user or process.
* A crash or code path reachable from fetched content (surah text, tafsir,
  audio) that could be turned into code execution.
* Credential or key handling: `UMMAH_API_KEY` is read from the environment and
  must never be written to the repository, logs, or the cache.

## What is out of scope

* **The content itself.** The Qur'anic text, recitations and tafsir come from
  third parties (EQuran.id, UmmahAPI). A wrong translation or a bad recitation is
  an upstream data issue, not a vulnerability here.
* **The speech model.** Supertonic 3 is a separate project with its own security
  process.
* **Missing hardening on a machine you do not control.** There is no privilege
  boundary in the app by design; anyone who can run it as your user can read your
  cache.

## How secrets are handled today

* There is no stored credential. The only optional secret is `UMMAH_API_KEY`,
  read from the environment at runtime.
* The cache and settings live under the standard per-user directories
  (`~/.cache/equran-cli`, `~/.local/share/equran-cli`) and inherit the user's
  filesystem permissions.
* Narration and the search index are built locally; no reading data leaves the
  machine.
