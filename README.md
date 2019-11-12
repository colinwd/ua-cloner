# UA Repository Cloner

Usage:

Set the `GITHUB_TOKEN` environment variable to your Github personal access token. You can generate a new one [here](https://github.com/settings/tokens).

```
ua-cloner <clone|update>
```

`clone` will ignore any directories that already exist. `update` will attempt to get a git branch in all first-level subdirectories of the directory it was launched in using `git rev-pare --abbrev-ref HEAD`, and ignore that subdirectory if the command errors or the current branch is not `master`.
