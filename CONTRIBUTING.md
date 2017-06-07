# Welcome

Our goal is to encourage frictionless contributions to the project. In order to achieve that, we use the Ethereum Classic [20-C4 RFC](https://etcrfc.that.world/20-C4/), which is derived from this project [Unprotocols](https://rfc.unprotocols.org). Our goal is to encourage contributions and have better code quality for SputnikVM.

In a nutshell, this means:

* We use optimistic merging to remove burden of the contributors. Your PRs are expected to be merged quickly.
* We encourage you to open a discussion issue first if you plan to implement a new feature that potentially change a large portion of the codebase.
* We do not require you to understand C4 to start contributing to SputnikVM.

# Submitting an issue

According to [development process](https://etcrfc.that.world/20-C4#24-development-process), the issue described should be documented and provable. What this means is that an issue should strive to have a clear, understandable problem statement.

Usually, only the creator of the issue are allowed to change the title and description, or change the issue status (open or closed). Maintainers are SHOULD only close other people's issue if he or she does not hear back from the creator. Maintainers MUST NOT modify other people's issue simply because of his or her preference of styles. Changing other contributor's issue without explaination is considered a mis-behaviour and SHOULD be avoided.

# Preparing a patch

According to [patch requirements](https://etcrfc.that.world/20-C4#23-patch-requirements), the patch should be a minimal and accurate answer to exactly one identified and agreed problem. Exceptions documented in 20-C4 should also be noted, however. A patch commit message should consist of a single short (less than 50 characters) line stating the problem ("Problem: ...") being solved, followed by a blank line and then the proposed solution ("Solution: ...").

```
Problem: short problem statement

Optional longer explanation of the problem that this patch
addresses, giving necessary details for the reader to be
able to understand it better.

Solution: explanation of the solution to the problem. Could
be longer than one line.
```

Also, please don't run `rustfmt` (`cargo fmt`) over your patch before committing. Otherwise, it'll make this patch unnecessarily long and might interfere with currently outstanding PRs or other items in progress. Instead please follow the [STYLEGUIDE](GUIDE.md).

# Commit logs and C4

Additional commit messages "tlogs" (as in "weblog => blog", "gitlog => tlog") are encouraged. It's a commit without changes to any files and it retains a contextualized article on the subject.

The motivation for this is that the web is not a reliable place to retain articles (hosts go down, content gets deleted, etc.). Nor is it easy to find relevant pieces with all the noise out there.

What did the contributor think about when he was developing this or that part? What train of thought was he on?

Keeping the articles in the git log allows us to retain them forever (for as long as there's at least one copy of the repository somewhere) and provide context to those who really want to learn more about the project.

It is highly recommended to watch [Pieter Hintjens' talk on building open source communities](https://www.youtube.com/watch?v=uzxcILudFWM) as well as read his [book on the same matter](https://www.gitbook.com/book/hintjens/social-architecture/details).
