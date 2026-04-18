# karakeep-sync

A tool to sync links from various services to [Karakeep](https://github.com/hoarder-app/hoarder) to keep all your interesting links in one place.

## Overview

When looking up something interesting you found in the past, you probably check multiple places - Karakeep, HN upvotes, Reddit bookmarks, etc. This tool syncs all those links to Karakeep automatically, organizing them under lists for easy future access.

## Supported Services

- ✅ Hacker News upvotes
- ✅ Reddit saved posts
- ✅ Github stars
- ✅ Pinboard bookmarks
- 🚧 X bookmarks (planned)
- 🚧 Bluesky bookmarks (planned)

## Environment Variables

Configure these environment variables in your `docker-compose.yml`:

| Variable           | Required | Description                                                       |
| ------------------ | -------- | ----------------------------------------------------------------- |
| `KS_KARAKEEP_AUTH` | ✅       | Your Karakeep API token                                           |
| `KS_KARAKEEP_URL`  | ✅       | Your Karakeep instance URL (e.g., `https://karakeep.example.com`) |

### For Hacker News

| Variable         | Required | Description                                      |
| ---------------- | -------- | ------------------------------------------------ |
| `KS_HN_AUTH`     | ❌       | Your Hacker News authentication cookie value     |
| `KS_HN_SCHEDULE` | ❌       | Sync schedule in cron format (default: `@daily`) |

Hacker news auth cookie can be obtained by logging into your HN account and inspecting the cookies in your browser. Look for the `user` cookie.

Hacker News upvotes will be synced to a list named `HN Upvoted` in your Karakeep instance.

Hacker News sync will be skipped if `KS_HN_AUTH` is not set.

### For Reddit

| Variable                 | Required | Description                                      |
| ------------------------ | -------- | ------------------------------------------------ |
| `KS_REDDIT_CLIENTID`     | ❌        | Your Reddit app client ID                        |
| `KS_REDDIT_CLIENTSECRET` | ❌        | Your Reddit app client secret                    |
| `KS_REDDIT_REFRESHTOKEN` | ❌        | Your Reddit app refresh token                    |
| `KS_REDDIT_USERNAME`     | ❌        | Your Reddit username (without `u/` prefix)       |
| `KS_REDDIT_SCHEDULE`     | ❌        | Sync schedule in cron format (default: `@daily`) |


To obtain a refresh token, you can follow these steps:

1. Create a Reddit app [here](https://www.reddit.com/prefs/apps) (choose "script" as the app type).
2. You can use a tool like [this](https://github.com/not-an-aardvark/reddit-oauth-helper) to generate a refresh token using your app's client ID and client secret. Make sure that the redirect URI matches the one provided from reddit-oauth-helper.
3. Make sure to give the app `history` scope access.
4. Make sure to tick the "permanent" option to get a refresh token.

If you don't want to trust a third party tool, you can also implement the OAuth2 flow yourself using the [Reddit API docs](https://www.reddit.com/dev/api/), or [manually generate a token](/REDDIT_REFRESH_TOKEN.md).

Reddit saves will be synced to a list named `Reddit Saved` in your Karakeep instance.

Reddit sync will be skipped if any of the following are not set:
- `KS_REDDIT_CLIENTID`
- `KS_REDDIT_CLIENTSECRET`
- `KS_REDDIT_REFRESHTOKEN`
- `KS_REDDIT_USERNAME`


### GitHub Stars

| Variable             | Required | Description                                      |
| -------------------- | -------- | ------------------------------------------------ |
| `KS_GITHUB_TOKEN`    | ❌       | Your GitHub personal access token                |
| `KS_GITHUB_SCHEDULE` | ❌       | Sync schedule in cron format (default: `@daily`) |

To obtain a GitHub personal access token, you can visit [this link](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens#creating-a-fine-grained-personal-access-token) and create a new token with `Starring` user permission (read).

GitHub stars will be synced to a list named `GitHub Starred` in your Karakeep instance.

GitHub sync will be skipped if `KS_GITHUB_TOKEN` is not set.

### Pinboard Bookmarks

| Variable               | Required | Description                                      |
| ---------------------- | -------- | ------------------------------------------------ |
| `KS_PINBOARD_TOKEN`    | ❌       | Your Pinboard API token                          |
| `KS_PINBOARD_SCHEDULE` | ❌       | Sync schedule in cron format (default: `@daily`) |

To obtain your Pinboard API token, visit your [Pinboard password page](https://pinboard.in/settings/password) and scroll down to the "API Token" section. The token will be in the format `username:TOKEN`.

Pinboard bookmarks will be synced to a list named `Pinboard` in your Karakeep instance.

Pinboard sync will be skipped if `KS_PINBOARD_TOKEN` is not set.

## Deployment

Create a `docker-compose.yml` file with the following content:

```yaml
services:
  karakeep-sync:
    image: ghcr.io/sidoshi/karakeep-sync:latest
    container_name: karakeep-sync
    restart: unless-stopped
    environment:
      - KS_KARAKEEP_AUTH=<your_karakeep_auth_cookie> # required
      - KS_KARAKEEP_URL=<your_karakeep_instance_url> # required

      - KS_HN_AUTH=<your_hn_auth_cookie> # optional
      - KS_HN_SCHEDULE=@daily # optional Cron format, e.g., "@hourly", "@daily", "0 0 * * *" default is "@daily"

      - KS_REDDIT_CLIENTID=<your_reddit_client_id> # optional
      - KS_REDDIT_CLIENTSECRET=<your_reddit_client_secret> # optional
      - KS_REDDIT_REFRESHTOKEN=<your_reddit_refresh_token> # optional
      - KS_REDDIT_USERNAME=<your_reddit_username> #optional
      - KS_REDDIT_SCHEDULE=@daily # optional Cron format, e.g., "@hourly", "@daily", "0 0 * * *" default is "@daily"

      - KS_GITHUB_TOKEN=<your_github_personal_access_token> # optional
      - KS_GITHUB_SCHEDULE=@daily # optional Cron format, e.g., "@hourly", "@daily", "0 0 * * *" default is "@daily"

      - KS_PINBOARD_TOKEN=<your_pinboard_api_token> # optional
      - KS_PINBOARD_SCHEDULE=@daily # optional Cron format, e.g., "@hourly", "@daily", "0 0 * * *" default is "@daily"
```

Then run:

```bash
docker-compose up -d
```

You can also add this service definition alongside your existing Hoarder/Karakeep services.

## Contributing

Contributions are welcome! Please open issues or pull requests for any features, bug fixes, or improvements.

To add support for more services, implement the `Plugin` trait in a new module under `crates/sync/src/plugin/`. You can refer to the existing `hn_upvotes` and `reddit_saves` modules as examples. All plugins must be registered in `crates/sync/src/plugin.rs`. Make sure to add appropriate configuration options in `crates/sync/src/settings.rs`. Finally, update the documentation in this README to include the new service.

See this PR for adding GitHub stars support as an example: [#2](https://github.com/sidoshi/karakeep-sync/pull/2)

## License

MIT License. See `LICENSE` file for details.
