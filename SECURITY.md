# Vulnerability Disclosure and Embargo Policy

The Open Device Partnership project welcomes the responsible disclosure of vulnerabilities.

## Initial Contact

All security bugs in Open Device Partnership should be reported to the security team.
To do so, please reach out in the form of a
[Github Security Advisory](https://docs.github.com/en/code-security/security-advisories/guidance-on-reporting-and-writing-information-about-vulnerabilities).

You will be invited to join this private area to discuss specifics. Doing so
allows us to start with a high level of confidentiality and relax it if the
issue is less critical, moving to work on the fix in the open.

Your initial contact will be acknowledged within 48 hours, and you’ll receive
a more detailed response within 96 hours indicating the next steps in handling
your report.

After the initial reply to your report, the security team will endeavor to
keep you informed of the progress being made towards a fix and full
announcement. As recommended by
[RFPolicy](https://dl.packetstormsecurity.net/papers/general/rfpolicy-2.0.txt),
these updates will be sent at least every five working days.

## Disclosure Policy

The Open Device Partnership project has a 5 step disclosure process.

1. Contact is established, a private channel created, and the security report
   is received and is assigned a primary handler. This person will coordinate
   the fix and release process.
2. The problem is confirmed and a list of all affected versions is determined.
   If an embargo is needed (see below), details of the embargo are decided.
3. Code is audited to find any potential similar problems.
4. Fixes are prepared for all releases which are still under maintenance. In
   case of embargo, these fixes are not committed to the public repository but
   rather held in a private fork pending the announcement.
5. The changes are pushed to the public repository and new builds are deployed.

This process can take some time, especially when coordination is required
with maintainers of other projects. Every effort will be made to handle the bug
in as timely a manner as possible, however it is important that we follow the
release process above to ensure that the disclosure is handled in a consistent
manner.

## Embargoes

While the Open Device Partnership project aims to follow the highest standards of
transparency and openness, handling some security issues may pose such an
immediate threat to various stakeholders and require coordination between
various actors that it cannot be made immediately public.

In this case, security issues will fall under an embargo.

An embargo can be called for in various cases:

- when disclosing the issue without simultaneously providing a mitigation
  would seriously endanger users,
- when producing a fix requires coordinating between multiple actors (such as
  upstream or downstream/dependency projects), or simply
- when proper analysis of the issue and its ramifications demands time.

If we determine that an issue you report requires an embargo, we will discuss
this with you and try to find a reasonable expiry date (aka “embargo
completion date”), as well as who should be included in the list of
need-to-know people.