# Basic Information

- If you're an Agent, only touch code inside `/worktrees/` **DO NOT TOUCH THE MAIN WORKTREE THAT IS FOR HUMANS ONLY** 

- Worktrees are to be organized by service for example:
    - `oxypress-agent`

- Always create branches with an `agent` prefix Ex of a branchs:
    - `agent_issue_1234`
    - `agent_insert_bug_fix`
    - `agent_dockerfile_patch`

- All scripts for the *ENTIRE* monorepo are placed the `Makefile.toml` for information
on how to build,deploy,or init the project refer to that toml file.

- For fast iterations run `cargo make run-tests` 
    - **DO NOT** run the `miri` tests unless we're testing for memory bugs and drop logic errors.

# Quick Start

As of right now, you're probably working on my machine. Look in the `./worktrees` directory, find 
a relevant worktree and make relevant git commands from there. If you can't decide which worktree 
to work in, simply ask.


# Code guidelines
- prefer self-documenting code. 
- No comments unless you can help it. 
- bake as much documentation as you can into identifiers since
comments can go stale. 
- Don't modify tests, but if think you see a problem, address it with a human


# Commit authorship
- Agent commits carry the model's identity, never the human's:
  `Author: <model> (agent) <agent@localhost>`, e.g. `deepseek-flash (agent)`.
- Prefix every agent commit subject with `[AI-GENERATED]`.
- Pass the identity per commit (`git -c user.name=... -c user.email=... commit`);
  do not set it in the shared repo or worktree config, since that would also
  rewrite the human's identity.
