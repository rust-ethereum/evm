# Welcome

Our goal is to encourage frictionless contributions to the project. In order to achieve that, we use the Ethereum Classic [1/C4 RFC](https://etcrfc.that.world/spec:1/C4), which is derived from this project [Unprotocols](https://rfc.unprotocols.org). Please read it, it will answer a lot of questions. Our goal is to merge pull requests as quickly as possible and make new stable releases regularly.

In a nutshell, this means:

* We merge pull requests rapidly (try!)
* We are open to diverse ideas
* We prefer code now over consensus later

Additional commit messages "tlogs" (as in "weblog => blog", "gitlog => tlog") are encouraged. It's a commit without changes to any files and it retains a contextualized article on the subject.

The motivation for this is that the web is not a reliable place to retain articles (hosts go down, content gets deleted, etc.). Nor is it easy to find relevant pieces with all the noise out there.

What did the contributor think about when he was developing this or that part? What train of thought was he on?

Keeping the articles in the git log allows us to retain them forever (for as long as there's at least one copy of the repository somewhere) and provide context to those who really want to learn more about the project.

It is highly recommended to watch [Pieter Hintjens' talk on building open source communities](https://www.youtube.com/watch?v=uzxcILudFWM) as well as read his [book on the same matter](https://www.gitbook.com/book/hintjens/social-architecture/details).

# Submitting an issue

According to [development process](https://etcrfc.that.world/spec:1/C4#24-development-process), the issue described should be documented and provable. What this means is that an issue should strive to have a clear, understandable problem statement. Just like a patch, it SHOULD be titled "Problem: ..." and have a detailed description describing evidence behind it, be it a bug or a feature request, or a longer term "exploratory" issue.

# Preparing a patch

According to [patch requirements](https://etcrfc.that.world/spec:1/C4#23-patch-requirements), the patch should be a minimal and accurate answer to exactly one identified and agreed problem. A patch commit message must consist of a single short (less than 50 characters) line stating the problem ("Problem: ...") being solved, followed by a blank line and then the proposed solution ("Solution: ...").

```
Problem: short problem statement

Optional longer explanation of the problem that this patch
addresses, giving necessary details for the reader to be
able to understand it better.

Solution: explanation of the solution to the problem. Could
be longer than one line.
```

We will even merge sloppy patches as we want this in the git history as evidence of bad actors. This sloppy code will be reverted or quickly replaced with fixes.

Also, please don't run `rustfmt` (`cargo fmt`) over your patch before committing. Otherwise, it'll make this patch unnecessarily long and might interfere with currently outstanding PRs or other items in progress. Instead please follow the [STYLEGUIDE](GUIDE.md).

# Specifications

Specifications RFCs of the Ethereum Classic Blockchain and thus SputnikVM are governed by [2/COSS RFC](https://etcrfc.that.world/spec:2/COSS).
