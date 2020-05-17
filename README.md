# issuefer

This program finds all TODOs in the source code and reports them as issues to GitHub or GitLab.

## What it does

1. Write a TODO somewhere in your source code in the form
```
// TODO: Some test
```
2. Run `issuefer -r` which will present you with the opportunity to automatically create a issue out of the found TODOs.
3. Issuefer will add the assigned issue number to the TODO and will automatically create a commit (it will not automatically push, you have to do that yourself).

## How to build it

Build it in the standard Rust way with
```
cargo build --release
```
The resulting binary can be found at `target/release/issuefer`.

## GitHub token

Issuefer needs a GitHub token to work. You can find out how to create [here](https://help.github.com/en/github/authenticating-to-github/creating-a-personal-access-token-for-the-command-line).

It will read the token from the environment variable `GITHUB_TOKEN`, so you can either run issuefer with
```bash
export GITHUB_TOKEN=YOUR_TOKEN /path/to/issuefer
```

or the easier way (maybe not the safest) is to add it to your ~/.profile:
```bash
echo 'export GITHUB_TOKEN="YOUR_TOKEN"' >> ~/.profile
```
You have to login again to apply that change.

## GitLab token

Issuefer needs a GitLab token when working with a GitLab repo. Create it on your GitLab page and set it with

```bash
export GITHUB_TOKEN=YOUR_TOKEN /path/to/issuefer
```

or with
```bash
echo 'export GITLAB_TOKEN="YOUR_TOKEN"' >> ~/.profile
```
to make it permanent, as it is done for GitHub.

In addition, if you are working with a custom GitLab installation you can specify the host name via

```bash
export GITLAB_HOST="your.gitlab.inst"
```

## Run it

When typing

```bash
./issuefer
```

it will report the untracked TODOs and TODOs where the corresponding issue has been closed.

To actually report new TODOs type
```bash
./issuefer -r
```

and to cleanup the sources from TODOs which correspond to already closed issues type
```bash
./issuefer -c
```

## Supported TODO formats

Currently issuefer only supports TODOs in the format
```CPP
// TODO: some text
```

and they have the stand in a separate line (with optional whitespaces/tabs in front).

In the future we will hopefully support also C like comments (`/* */`) and multi line TODOs (which will then add the additional lines as body to the issue).