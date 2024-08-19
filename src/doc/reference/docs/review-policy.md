Team members are given permission to merge changes from other contributors in the <https://github.com/rust-lang/reference> repository. There are different guidelines for reviewing based on the kind of changes being made:

## Policy changes

- Significant changes to the policy of how the team operates, such as changes to this document, should have agreement of the team without any blocking objections.
- Minor changes to something like the style enforcement can be made with the review of a team member, as long as there is high confidence that it is unlikely any team member would object (for example, codifying a guideline that is already in practice), and that the change can be easily reversed.

## Meaningful content addition or changes

- When adding or changing content in the spec, the reviewer should consult with appropriate experts to validate the changes. This may not be required if the reviewer has high confidence that the changes are correct, and consider themselves well-versed enough in the topic to understand it, or the relevant experts are the author or have been heavily involved in the process. It is up to the reviewer to use their best judgement when to consult.
- Content should always follow the guidelines from the [authoring guide].

## Minor content changes
- Minor content changes, such as small cleanups or wording fixes, can be made with the review from a team member without further consultation.

## Tooling changes
- Minor changes to the tooling may be made with a review from a team member. This includes bug fixes, minor additions that are unlikely to have objections, and additions that have already been discussed.
- Major changes, such as a change in how content is authored, or major changes to how the tooling works should be approved by the team without blocking objections.

## Review Process Flowchart

When reviewing a pull request, ask yourself the following questions:

### Are the proposed changes true?

If we're not sure and can't easily verify it ourselves, we ask someone who would know.

### Does this make any new guarantees about the language?

If this would make a new guarantee about the language, this needs to go through the `lang` team to be accepted (unless the `lang` team has clearly accepted this guarantee elsewhere). Ask @traviscross or @pnkfelix if at all unsure about any of these.

### Would we have added this to the Reference ourselves?

There are a number of PRs that might be true, but when we look at them, we think to ourselves, in our heart of hearts, that this just isn't something we would have bothered to write ourselves. We don't want to accept a PR just because it's in front of us and not obviously false. It should clearly add value.

### Is this editorially sound?

Some PRs try to "sell" the language too much, or try to explain more (or less) than needed, or give too many (or too few) examples, etc. The PR should match the general flavor of the Reference here.

### Is this well written? 

Some PRs are right but are awkwardly worded or have typographical problems. If the changes are small, we'll just add commits to the branch to clean things up, then merge.

<!-- TODO -->
This policy does not yet cover the process for getting final approval from the relevant teams.

[authoring guide]: authoring.md