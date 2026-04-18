# Manually Generating a Reddit Refresh Token

Obtain your Reddit app's `client ID` and `secret` from the [developed applications](https://www.reddit.com/prefs/apps) section of your account preferences (or from [Old Reddit](https://old.reddit.com/prefs/apps)).

Paste your `client ID` into the following path:
```plaintext
https://www.reddit.com/api/v1/authorize?client_id=<CLIENT_ID>&response_type=code&state=debug&redirect_uri=http://localhost&duration=permanent&scope=history%20read%20save
```

> [!IMPORTANT]
> The `redirect_uri` must match your Reddit app configuration

Paste the URI into a web browser. It should prompt you to allow access. After granting access, the page will return an error message.

Copy the URI out of the address bar, it will look like this:
```plaintext
http://localhost/?state=debug&code=<AUTH_CODE>#_
```

Copy the `auth code` out of the error URI (remove the `#_` suffix).

Construct the following CURL command:
```bash
curl -X POST "https://www.reddit.com/api/v1/access_token" \
  -u "<CLIENT_ID>:<CLIENT_SECRET>" \
  -H "User-Agent: karakeep-sync/1.0 by sidoshi" \
  -d "grant_type=authorization_code" \
  -d "code=<AUTH_CODE>" \
  -d "redirect_uri=http://localhost"
```

Run the CURL command; you should receive a response like this:
```json
{
    "access_token": "######",
    "token_type": "bearer",
    "expires_in": 86400,
    "refresh_token": "######",
    "scope": "read history"
}
```

Extract the `refresh_token`, this is your `KS_REDDIT_REFRESHTOKEN` in Karakeep-sync.
