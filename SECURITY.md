# Security Policy

## Supported Versions

As Worknest is currently in early development (pre-1.0), we will provide security
updates for the latest release only.

| Version | Supported          |
| ------- | ------------------ |
| 0.x.x   | :white_check_mark: |

Once we reach 1.0, we will maintain security updates for the current major version
and the previous major version.

## Reporting a Vulnerability

We take the security of Worknest seriously. If you believe you have found a
security vulnerability, please report it to us as described below.

### Please Do Not

- Open a public GitHub issue for security vulnerabilities
- Disclose the vulnerability publicly before it has been addressed

### Please Do

1. **Email us** at security@worknest.dev (or create a private security advisory on GitHub)
2. **Provide detailed information** including:
   - Type of vulnerability
   - Full paths of affected source files
   - Location of the affected source code (tag/branch/commit or direct URL)
   - Step-by-step instructions to reproduce the issue
   - Proof-of-concept or exploit code (if possible)
   - Impact of the vulnerability

### What to Expect

- **Acknowledgment**: We will acknowledge receipt of your vulnerability report within
  48 hours.
- **Communication**: We will keep you informed about our progress toward resolving
  the issue.
- **Timeline**: We aim to address critical vulnerabilities within 7 days and other
  vulnerabilities within 30 days.
- **Credit**: We will credit you in the security advisory unless you wish to remain
  anonymous.

## Security Update Process

When we receive a security report:

1. We will confirm the vulnerability and determine its severity
2. We will develop a fix
3. We will prepare a security advisory
4. We will release a patched version
5. We will publish the security advisory

## Security Best Practices for Users

### Data Security

- **Database Encryption**: Your SQLite database is stored locally. Ensure your
  system uses full-disk encryption for sensitive data.
- **Passwords**: Strong passwords are enforced, but ensure you use unique passwords
  for each service.
- **Backups**: Regularly backup your database file (default location: `~/.worknest/`)

### System Security

- **Keep Updated**: Always use the latest version of Worknest to benefit from
  security patches.
- **Audit Dependencies**: Run `cargo audit` to check for known vulnerabilities
  in dependencies.
- **Plugins** (future): Only install plugins from trusted sources. Review plugin
  permissions before granting access.

### Network Security (future cloud sync)

- **HTTPS Only**: We will only support HTTPS for cloud sync.
- **Token Management**: Keep your authentication tokens secure. Never share them.
- **Logout**: When done, properly logout to invalidate session tokens.

## Security Features

### Current (MVP)

- Password hashing with bcrypt (cost factor: 12)
- JWT-based authentication with expiration
- Input validation and sanitization
- Parameterized SQL queries (SQL injection prevention)

### Planned

- **v2.0**: Role-based access control (RBAC)
- **v3.0**: Plugin sandboxing with WASM
- **v4.0**: End-to-end encryption for cloud sync
- **v4.0**: SSO/SAML integration
- **v4.0**: Audit logging

## Third-Party Dependencies

We regularly audit our dependencies for known vulnerabilities using:

- `cargo audit` in our CI/CD pipeline
- Dependabot for automated dependency updates
- Manual review of critical dependencies

## Disclosure Policy

We practice coordinated vulnerability disclosure:

- We will work with security researchers to understand and fix the issue
- We will publicly disclose the vulnerability after a fix is released
- We will credit the reporter (unless they wish to remain anonymous)

## Security Hall of Fame

We will maintain a list of security researchers who have responsibly disclosed
vulnerabilities to us. Thank you to:

<!-- This section will be updated as researchers report vulnerabilities -->

*No vulnerabilities reported yet (project in early development)*

## Contact

For security concerns, please contact:

- **Email**: security@worknest.dev (or create a GitHub Security Advisory)
- **PGP Key**: (will be provided when project is more mature)

---

Thank you for helping keep Worknest and its users safe!
