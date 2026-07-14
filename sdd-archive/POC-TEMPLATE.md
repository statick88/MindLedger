# PoC Documentation Template

Use this template for each security finding. Copy and fill in for every CRITICAL/HIGH/MEDIUM finding.

---

## Finding: [FINDING-ID] — [Short Title]

### Classification

| Field | Value |
|-------|-------|
| **Severity** | CRITICAL / HIGH / MEDIUM / LOW / INFO |
| **CVSS v4.0** | [Vector String] = [Score] |
| **OWASP Category** | A03:2021 Injection / A05:2021 Misconfig / A06:2021 Vuln Components |
| **MITRE CWE** | CWE-89 (SQL Injection) / CWE-79 (XSS) / etc. |
| **Domain** | IPC Injection / SQLCipher / Dependencies / Business Logic / SAST |
| **Status** | Open / Mitigated / Accepted / False Positive |

### Description

[Clear, concise description of the vulnerability and its impact.]

### Affected Component

| Field | Value |
|-------|-------|
| **File** | `path/to/affected/file.rs` |
| **Function** | `function_name()` |
| **Line(s)** | L42-L58 |

### Reproduction Steps (PoC)

```
1. [Step 1 — setup or precondition]
2. [Step 2 — action to trigger]
3. [Step 3 — observe vulnerability]
```

**Expected (vulnerable) behavior**: [What happens without the fix]
**Actual (secure) behavior**: [What should happen / what the fix achieves]

### Evidence

```
[Paste relevant output, test result, or log excerpt]
```

### Impact Analysis

- **Confidentiality**: [None / Low / High] — [explanation]
- **Integrity**: [None / Low / High] — [explanation]
- **Availability**: [None / Low / High] — [explanation]
- **Data at risk**: [PHI / financial data / credentials / none]

### Remediation

| Field | Value |
|-------|-------|
| **Recommendation** | [Specific fix recommendation] |
| **Effort** | [S / M / L / XL] |
| **Priority** | P0 / P1 / P2 / P3 |
| **Owner** | [Team or individual] |
| **Deadline** | [Date or "before production deploy"] |

### References

- OWASP: [link to relevant OWASP page]
- MITRE: [link to CWE entry]
- CVE: [CVE ID if applicable]
- Internal: [link to issue tracker, if any]

### Verification

- [ ] Fix implemented
- [ ] Test written and passing
- [ ] Regression test added
- [ ] Reviewed by second engineer

---

## Quick Reference: CVSS v4.0 Base Score Calculation

For findings that need scoring, use the CVSS v4.0 calculator:
https://www.first.org/cvss/calculator/4.0

### Common Vectors for MindLdger

| Scenario | Typical Vector |
|----------|---------------|
| SQL injection via IPC | AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:N |
| XSS in clinical notes | AV:N/AC:L/PR:N/UI:R/S:C/C:L/I:L/A:N |
| Hardcoded credentials | AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:N |
| Missing input validation | AV:N/AC:L/PR:N/UI:N/S:U/C:N/I:L/A:N |
| Information disclosure | AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:N/A:N |
| Denial of service | AV:N/AC:L/PR:N/UI:N/S:U/C:N/I:N/A:H |
