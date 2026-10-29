# GitHub mirror templates

Dwall downloads themes, thumbnails and update manifests from GitHub. A mirror
template rewrites those URLs through a host that can reach GitHub when the direct
connection cannot.

## Placeholders

| Placeholder | Meaning |
| --- | --- |
| `<owner>` | GitHub owner, for example `dwall-rs` |
| `<repo>` | Repository name, for example `dwall-assets` |
| `<version>` | Release tag, or `latest` for the stable update route |
| `<asset>` | Release asset name, for example `Big Sur.zip` |

A template must contain `<owner>` and `<repo>`. Release downloads also need
`<asset>`; without it the resolver keeps the original GitHub URL instead of
building a URL that is guaranteed to 404.

## Examples

Host-swapping proxy for thumbnails and raw files:

```text
https://ghproxy.example/https://github.com/<owner>/<repo>
```

Release-asset proxy:

```text
https://ghproxy.example/https://github.com/<owner>/<repo>/releases/download/<version>/<asset>
```

A template that only swaps the host (for example `https://kkgithub.com/<owner>/<repo>`)
works for raw files but cannot express a release asset, so theme downloads and update
checks fall back to GitHub.

The settings window stores this template in `config.toml` as the string value of
`network`; an empty value is the same as no mirror.
