# Security Policy

## Reporting a Vulnerability

The security of Dweve systems and our users is our highest priority. If you believe you have found a security vulnerability in any Dweve-owned repository, please report it to us as described below.

### How to Report

**DO NOT** create a public GitHub issue for security vulnerabilities.

Instead, please report security vulnerabilities by emailing:

**security@dweve.com**

You should receive a response within 24 hours. If for some reason you do not, please follow up via email to ensure we received your original message.

### What to Include

Please include the following information in your report:

- Type of issue (e.g., buffer overflow, SQL injection, cross-site scripting, etc.)
- Full paths of source file(s) related to the manifestation of the issue
- The location of the affected source code (tag/branch/commit or direct URL)
- Any special configuration required to reproduce the issue
- Step-by-step instructions to reproduce the issue
- Proof-of-concept or exploit code (if possible)
- Impact of the issue, including how an attacker might exploit it

### Encrypted Communication

For sensitive security reports requiring encrypted communication, contact security@dweve.com to request our PGP public key.

## Our Commitment

When you report a security issue, we commit to:

1. **Acknowledge** your report within 24 hours
2. **Investigate** the issue and keep you informed of our progress
3. **Fix** confirmed vulnerabilities in a timely manner
4. **Credit** you (if desired) in our security advisories
5. **Never** take legal action against good-faith security researchers

## Scope

### In Scope

- All Dweve-owned repositories
- Dweve platform (platform.dweve.com)
- Dweve APIs
- Dweve Mesh infrastructure
- dweve.com and subdomains
- docs.dweve.com

### Out of Scope

- Third-party applications using Dweve APIs
- Vulnerabilities in third-party libraries (report to the library maintainer)
- Social engineering attacks
- Denial of service attacks
- Spam or social media account takeovers
- Issues requiring physical access to a user's device

## Safe Harbor

We consider security research conducted in accordance with this policy to be:

- Authorized concerning any applicable anti-hacking laws
- Authorized concerning any relevant anti-circumvention laws
- Exempt from restrictions in our Terms of Service that would interfere with conducting security research

We will not pursue legal action against researchers who:

- Make a good faith effort to avoid privacy violations, destruction of data, and interruption of services
- Only interact with accounts they own or with explicit permission of account holders
- Do not exploit vulnerabilities beyond what is necessary to demonstrate the issue
- Report vulnerabilities promptly after discovery
- Give us reasonable time to address issues before public disclosure

## Supported Versions

We provide security updates for the following versions:

| Version | Supported          |
| ------- | ------------------ |
| Latest  | :white_check_mark: |
| Latest - 1 minor | :white_check_mark: |
| Older   | :x:                |

For enterprise customers with extended support agreements, additional versions may be covered.

## Security Best Practices

When using Dweve software, we recommend:

1. **Keep software updated** to the latest stable version
2. **Use strong authentication** including MFA where available
3. **Follow the principle of least privilege** when configuring access
4. **Monitor logs** for suspicious activity
5. **Report anomalies** to security@dweve.com

## Disclosure Policy

- We aim to fix critical vulnerabilities within 7 days
- We aim to fix high-severity vulnerabilities within 30 days
- We will coordinate public disclosure with the reporter
- We typically request 90 days before public disclosure

## Recognition

We maintain a hall of fame for security researchers who have helped improve Dweve security:

https://dweve.com/security/hall-of-fame

Researchers who report valid vulnerabilities will be recognized (with their permission) and may be eligible for our bug bounty program.

## Bug Bounty Program

We offer rewards for qualifying vulnerability reports. Bounty amounts depend on severity:

| Severity | Bounty Range |
| -------- | ------------ |
| Critical | EUR 1,000 - 5,000 |
| High     | EUR 500 - 1,000 |
| Medium   | EUR 100 - 500 |
| Low      | EUR 50 - 100 |

Contact security@dweve.com for current program details and eligibility requirements.

## Contact

**Security Team**: security@dweve.com
**24/7 Emergency**: +31 (0)85 0041 022

Dweve B.V.
Meander 251
6825 MC Arnhem
The Netherlands

---

Made with love in Europe, for Europeans who believe technology should serve humanity, not the other way around.
