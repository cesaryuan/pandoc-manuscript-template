# Development

## Release to PyPI

This project publishes to PyPI with GitHub Actions trusted publishing, so release jobs do not need a stored PyPI token.

One-time setup:

1. Create the project on PyPI, or create a pending publisher if this is the first release.
2. In the GitHub repository, create an environment named `pypi`.
3. In the PyPI project settings, add a trusted publisher for this repository, the `publish-pypi.yml` workflow, and the `pypi` environment.

Pending publishers do not reserve the package name until the first successful publish, so run the first release soon after registering one.

Release steps:

```bash
uv build
git status --short
git tag v0.1.0
git push origin v0.1.0
```

The workflow builds the wheel and source distribution, smoke-tests both artifacts with `pmt --help`, then runs `uv publish`.
