# Security Policy

## Scope

This repository is a learning project for the experimental Topcoat 0.4
framework. It is not maintained as a production-ready application, and no
version currently receives production security support.

The latest `main` branch is the only version considered for fixes.

## Reporting a vulnerability

Please do not disclose a suspected vulnerability, credential, session token,
or personal data in a public issue.

Use GitHub's **Report a vulnerability** form in the repository's Security tab
to send a private report. Include the affected route or component, reproduction
steps, and the impact where possible.

## Known limitations

The sample does not implement rate limiting, email verification, password
reset, periodic expired-session cleanup, or a complete set of production
security headers. See `README.md` and `DEPLOYMENT.md` before adapting it.
