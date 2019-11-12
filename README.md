# UA Repository Cloner

Usage:

```
GITHUB_TOKEN="your github api token" ua-cloner <clone|update>
```

`clone` will ignore any directories that already exist. `update` will attempt to get a git branch in all first-level subdirectories of the directory it was launched in using `git rev-pare --abbrev-ref HEAD`, and ignore that subdirectory if the command errors or the current branch is not `master`.