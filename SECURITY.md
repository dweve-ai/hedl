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

## Our Commitment

When you report a security issue, we commit to:

1. **Acknowledge** your report within 24 hours
2. **Investigate** the issue and keep you informed of our progress
3. **Fix** confirmed vulnerabilities in a timely manner
4. **Credit** you (if desired) in our security advisories
5. **Never** take legal action against good-faith security researchers

## Scope

### In Scope

- The HEDL repository and its crates

### Out of Scope

- Vulnerabilities in third-party dependencies (report to the upstream maintainer)
- Denial of service attacks

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

## Disclosure Policy

- We aim to fix critical vulnerabilities within 7 days
- We aim to fix high-severity vulnerabilities within 30 days
- We will coordinate public disclosure with the reporter
- We typically request 90 days before public disclosure

## Contact

**Security Team**: security@dweve.com

Dweve B.V.
Meander 251
6825 MC Arnhem
The Netherlands
