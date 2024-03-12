# Motivational Overleaf word counter discord bot

This bot keeps us motivated writing overleaf documents, continuesly reminding us why we are doing what we do.

## Creating API secrets

To be able to run this code, you need a few environment variables. These are:

- `DISCORD_TOKEN`
- `DISCORD_CHANNEL`
- `OVERLEAF_SESSION_KEY`
- `OVERLEAF_DOC_ID`

These can either be a environment variable, or saved in `Secrets.toml`

Example toml:

```toml
DISCORD_TOKEN="<Discord bot token>"
DISCORD_CHANNEL="<The channel to send messages in>"
OVERLEAF_SESSION_KEY="<Overleaf session cookie>"
OVERLEAF_DOC_ID="<Overleaf id found in URL>"
```

The overleaf session key can be found in the `overleaf_session2` cookie.
