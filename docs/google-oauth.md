# Google OAuth Client Setup

This app needs a Google OAuth **Desktop app** client so it can request
read-only access to your Google Calendar from your own computer.

You need two values from Google Cloud:

- Client ID
- Client Secret

The app saves them locally as:

```text
~/.config/waybar-google-calendar/client_secret.json
```

The browser login token is created later and saved separately at:

```text
~/.local/share/waybar-google-calendar/oauth-token.json
```

## 1. Open Google Cloud

Open:

```text
https://console.cloud.google.com/auth/clients
```

Choose an existing personal project or create a new project such as
`Waybar Google Calendar`.

Google Cloud labels change over time. If the page shows **Google Auth
Platform**, **OAuth overview**, **Clients**, or **Credentials**, use the closest
matching option.

## 2. Enable Google Calendar API

Open:

```text
https://console.cloud.google.com/apis/library/calendar-json.googleapis.com
```

Make sure the selected project is the one you chose for this app, then click
**Enable**. If the button says **Manage**, the API is already enabled.

## 3. Configure OAuth Consent

If Google asks you to configure the OAuth consent screen or Google Auth
Platform:

- App name: `Waybar Google Calendar`
- User support email: your Google email
- Developer contact email: your Google email
- Audience/user type:
  - Use **External** for a personal Gmail account.
  - Use **Internal** only for a Google Workspace organization where that option
    is available.
- Test users: add the same Google account whose calendar you want to show.

The app requests this Google Calendar read-only scope during browser login:

```text
https://www.googleapis.com/auth/calendar.readonly
```

You usually do not need to manually add scopes while creating the client. If
Google asks for scopes, add the Calendar read-only scope above.

## 4. Create a Desktop OAuth Client

Go back to:

```text
https://console.cloud.google.com/auth/clients
```

Create a new OAuth client:

- Application type: **Desktop app**
- Name: `Waybar Google Calendar`

After creation, Google shows a **Client ID** and **Client Secret**. Copy both
values immediately and paste them into the app.

If you closed the result window, open the client details from the Clients page.
If the secret is no longer available, create a new Desktop app client and use
the new values.

## 5. Save and Authenticate

In the Waybar Google Calendar popup:

1. Paste the Client ID.
2. Paste the Client Secret.
3. Click **Save & Authenticate**.
4. Approve read-only Calendar access in the browser.

If Google shows an unverified-app warning, confirm that this is the private app
you created in your own Google Cloud project and continue with your test user
account.

## References

- Google Calendar API authorization guide:
  https://developers.google.com/calendar/api/guides/auth
- Google API Console OAuth setup help:
  https://support.google.com/googleapi/answer/6158849
