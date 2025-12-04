# Working with cargo vet

## Introduction

`cargo vet` is a tool to help ensure that third-party Rust dependencies have been audited by a trusted entity.
It matches all dependencies against a set of audits conducted by the authors of the project or entities they trust.  
To learn more, visit [mozilla/cargo-vet](https://github.com/mozilla/cargo-vet)

---

## Adding a new dependency

When updating or adding a new dependency, we need to ensure it's audited before being merged into main.  
For our repositories, we have designated experts who are responsible for vetting any new dependencies being added to their repository.  
_It is the shared responsibility of the developer creating the PR and the auditors to conduct a successful audit._  
Follow the process below to ensure compliance:

### For Developers
1. **Respond to `cargo vet` failures**:
  - If your PR fails the `cargo vet` step, the cargo-vet workflow will add a comment to the PR with a template questionnaire
  - Copy the questionnaire, fill it out and paste it as a new comment on the PR. This greatly helps the auditors get some context of the changes requiring the new dependencies

2. **Engage with auditors**:
  - Respond to any questions that the auditors might have regarding the need of any new dependencies

3. **Rebase and verify**:
  - At their discretion, auditors will check in their audits into either [rust-crate-audits](https://github.com/OpenDevicePartnership/rust-crate-audits) or into the same repository
  - Once the new audits have been merged, rebase your branch on main and verify it passes `cargo vet`
    ```bash
    git fetch upstream
    git rebase upstream/main
    cargo vet
    ```

4. **Update PR**:
  - If the audits were checked into rust-crate-audits, they will show up in _imports.lock_ on running `cargo vet`. In this case add the updated _imports.lock_ to your PR
  - If the audits were checked into the same repository, they will be present in _audits.toml_ after rebase and you can simply force push to your PR after rebase
    ```bash
    git push -f
    ```

5. **Check PR status**:
  - The existing PR comment from the previous failure will be updated with a success message once the check passes

### For Auditors

1. **Review the questionnaire**:
  - Check the filled questionnaire on the PR once the developer responds to the `cargo vet` failure
  - Respond to the developer comment in case more information is needed

2. **Audit new dependencies**:
  - Inspect the `cargo vet` failures using your preferred method
    - Use [gh pr checkout](https://cli.github.com/manual/gh_pr_checkout) to checkout the PR and run `cargo vet --locked`
    - Use [Github Pull Requests for Visual Studio Code](https://marketplace.visualstudio.com/items?itemName=GitHub.vscode-pull-request-github) to checkout the PR and run `cargo vet --locked`
    - For more suggestions: [Checking out pull requests locally](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/reviewing-changes-in-pull-requests/checking-out-pull-requests-locally)

3. **Follow `cargo vet` recommendations**:
  - Follow the recommendations of the `cargo vet` command output, either `cargo vet diff` for version update or `cargo vet inspect` for new dependencies

4. **Record audits**:
  - Use `cargo vet certify` to add new audits to _audits.toml_
  - Verify all dependencies pass using `cargo vet`

5. **Decide audit location**:
  - **Shared audits**: New audits should ideally be shared across ODP repositories to reduce the overhead of multiple audits for the same dependencies. To facilitate this, it's recommended to cut and paste the new audits and submit as a separate PR to the _audits.toml_ in [rust-crate-audits](https://github.com/OpenDevicePartnership/rust-crate-audits)
  - If due to business reasons, the audits are not to be shared across repositories, copy the updated _audits.toml_ to a new branch off main in the same repository and submit the PR to update the audits

6. **Communicate successful audit**:
  - Communicate to the PR developer via a PR comment so they can update the PR and get `cargo vet` to pass

---

## Audit criteria
`cargo vet` comes pre-equipped with two built-in criteria but supports adding new criteria to suit our needs.  
As defined [here](https://mozilla.github.io/cargo-vet/built-in-criteria.html), the default criteria are:

- **safe-to-run**  
  This crate can be compiled, run, and tested on a local workstation or in
  controlled automation without surprising consequences, such as:
  * Reading or writing data from sensitive or unrelated parts of the filesystem.
  * Installing software or reconfiguring the device.
  * Connecting to untrusted network endpoints.
  * Misuse of system resources (e.g. cryptocurrency mining).

- **safe-to-deploy**  
  This crate will not introduce a serious security vulnerability to production
  software exposed to untrusted input.
  
  Auditors are not required to perform a full logic review of the entire crate.
  Rather, they must review enough to fully reason about the behavior of all unsafe
  blocks and usage of powerful imports. For any reasonable usage of the crate in
  real-world software, an attacker must not be able to manipulate the runtime
  behavior of these sections in an exploitable or surprising way.
  
  Ideally, all unsafe code is fully sound, and ambient capabilities (e.g.
  filesystem access) are hardened against manipulation and consistent with the
  advertised behavior of the crate. However, some discretion is permitted. In such
  cases, the nature of the discretion should be recorded in the `notes` field of
  the audit record.
  
  For crates which generate deployed code (e.g. build dependencies or procedural
  macros), reasonable usage of the crate should output code which meets the above
  criteria.

  **Note: `safe-to-deploy` implies `safe-to-run`**

---

## Conducting an audit

When performing an audit for a new or updated dependency, auditors may consider the following criteria to ensure the safety, reliability, and suitability of the crate for use in our projects:

- **Security**:  
  - Review the crate for known vulnerabilities or security advisories.
  - Check for unsafe code usage and ensure it is justified and well-documented.
  - Evaluate the crate’s history of security issues and responsiveness to reported problems.

- **Maintenance and Activity**:  
  - Assess the frequency of updates and the responsiveness of maintainers to issues and pull requests.
  - Prefer crates that are actively maintained and have a healthy contributor base.

- **License Compliance**:  
  - Verify that the crate’s license is compatible with our project’s licensing requirements.

- **Community Trust and Adoption**:  
  - Consider the crate’s adoption in the wider Rust ecosystem.
  - Prefer crates that are widely used and trusted by the community.

- **Functionality and Suitability**:  
  - Confirm that the crate provides the required functionality without unnecessary features or bloat.
  - Evaluate whether the crate’s API is stable and unlikely to introduce breaking changes unexpectedly.

- **Audit Trail**:  
  - Record the audit decision, including any concerns, mitigations, or recommendations for future updates.
  - If exemptions are granted, document the justification and any follow-up actions required.

---

## Tips for using `cargo vet`:

- **Update _imports.lock_**:
  - Import trusted third party audits to reduce the number of new audits to be performed. Running `cargo vet` without `--locked` fetches new imports and updates _imports.lock_ with any audits that are helpful for our project.

- **Add exemptions**:
  - If an audit cannot be performed for some dependency due to time sensitivity or business justified reasons, use `cargo vet add-exemption <PACKAGE> <VERSION>` to add the dependency to exemptions in _config.toml_
  - To add all remaining audits to exemptions at once, use `cargo vet regenerate exemptions`

- **Prune unnecessary entries**:
  - Remove unnecessary exemptions and imports using `cargo vet prune`