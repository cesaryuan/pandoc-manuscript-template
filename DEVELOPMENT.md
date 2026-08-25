# Development

## Private native submodules

The `scripts/mathtype-rust` and `scripts/latex2wmf` source trees are private
submodules. A source checkout therefore requires GitHub read access to both
repositories:

```bash
git clone --recurse-submodules https://github.com/cesaryuan/pandoc-manuscript-template.git
```

For an existing checkout, initialize or refresh them with:

```bash
git submodule update --init --recursive
```

The GitHub Actions workflows use the repository secret
`PRIVATE_SUBMODULES_TOKEN`. Configure it with a least-privilege token that has
read-only Contents access to the parent repository and both private submodule
repositories. Secrets are not provided to workflows triggered by pull requests
from forks, so those runs cannot fetch the private source trees.

## Release to PyPI

This project publishes to PyPI with GitHub Actions trusted publishing, so release jobs do not need a stored PyPI token.

One-time setup:

1. Create the project on PyPI, or create a pending publisher if this is the first release.
2. In the GitHub repository, create an environment named `pypi`.
3. In the PyPI project settings, add a trusted publisher for this repository, the `publish-pypi.yml` workflow, and the `pypi` environment.

Pending publishers do not reserve the package name until the first successful publish, so run the first release soon after registering one.

Release steps:

```bash
git status --short
uvx bump-my-version bump patch
git push origin main --tags
```

Use `minor` or `major` instead of `patch` when appropriate. The version bump command updates `pyproject.toml` and the root package entry in `uv.lock`, creates a release commit, and tags it as `v{new_version}`. The workflow builds Windows, macOS 14-targeted, and manylinux wheels, smoke-tests their bundled native helpers, then runs `uv publish`.
