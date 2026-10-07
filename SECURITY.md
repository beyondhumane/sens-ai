# Security policy

## Supported versions

Only the latest release is fixed. Sens updates itself — *Settings › General › Updates* — so a fix ships as a new release, never as a patch to an old one.

| Version | Fixed |
| --- | --- |
| The [latest release](https://github.com/beyondhumane/sens-ai/releases/latest) | Yes |
| Anything older | No: update first, then check whether it still happens |

## Reporting a vulnerability

Do not open a public issue. Report it privately, through [GitHub's vulnerability reporting](https://github.com/beyondhumane/sens-ai/security/advisories/new) for this repository.

A report that can be acted on says:

- the Sens version (*Settings › General*) and the Windows version;
- what an attacker gains, and from where: a web page, a repository, a plugin, another program on the same computer;
- the steps, or a proof of concept, that show it;
- whether you have told anyone else.

The report stays private while it is confirmed and fixed. Once the release with the fix is out, the advisory is published with credit to you, unless you would rather not be named.

## Scope

In scope: what Sens itself does.

- The app and its engine: the commands the interface sends to `sens-app`, how sessions start `claude`, and the tools Sens offers it.
- The installer, the uninstaller and the updater, including the signature every update is checked against.
- What Sens downloads and checks: Claude Code against its checksum, the voice model against its SHA-256, plugins and skills at a pinned commit.
- The API key Sens seals with Windows DPAPI.
- The website, [sens.beyondhumane.com](https://sens.beyondhumane.com).

Out of scope:

- Claude Code and Anthropic's services. Report those to Anthropic, under its [responsible disclosure policy](https://www.anthropic.com/responsible-disclosure-policy).
- Third-party plugins, skills and MCP servers. Report those to their authors; Sens shows what each one runs before it is installed.
- What Claude does with a permission you gave it, such as the *No checks* mode.
- The SmartScreen warning on the installer, which is not code-signed yet. The README says how to check the download.
