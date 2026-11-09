# SumiLedger Backend API

> **Implementation audit:** This document is based on the uploaded backend source (`src/main.rs`, service handlers, service queries, and SQL migrations). It describes the code in that snapshot, not an independently tested production deployment.
>
> **Important current limitation:** Authentication is inconsistent between handler groups. Authentication, organization, and account handlers read the `session` cookie. Journal-entry and accounting-report handlers instead require `Authorization: Bearer <token>`. The login endpoint only exposes the token through an HttpOnly cookie, not in its JSON response. See [Authentication compatibility issue](#authentication-compatibility-issue).

## 1. Overview

- Base URL for local Docker deployment: `http://localhost:3000`
- Content type for JSON requests: `application/json`
- IDs: UUID strings
- Dates: ISO-8601 calendar dates (`YYYY-MM-DD`)
- Monetary values: decimal strings in JSON where returned by the API
- Maximum request body: 64 KiB
- Database: PostgreSQL; SQLx migrations run automatically at backend startup.

Set a base URL for the examples:

```bash
export API="http://localhost:3000"
```

## 2. Endpoint inventory

| Area | Method | Path | Authentication |
|---|---|---|---|
| Health | GET | `/health` | None |
| Auth | POST | `/auth/register` | None |
| Auth | POST | `/auth/login` | None |
| Auth | GET | `/auth/me` | Session cookie |
| Auth | POST | `/auth/logout` | Session cookie |
| Auth | POST | `/auth/logout-all` | Session cookie |
| Auth | POST | `/auth/change-pass` | Session cookie |
| Password reset | POST | `/auth/forgot-pass` | None |
| Password reset | POST | `/auth/reset-password` | None |
| Organizations | POST | `/organizations` | Session cookie |
| Organizations | GET | `/organizations` | Session cookie |
| Organizations | GET | `/organizations/{id}` | Session cookie |
| Organizations | POST | `/organizations/{id}/members` | Session cookie; owner/admin |
| Accounts | POST | `/organizations/{id}/accounts` | Session cookie |
| Accounts | GET | `/organizations/{id}/accounts` | Session cookie |
| Accounts | GET | `/organizations/{id}/accounts/{account_id}` | Session cookie |
| Journal entries | POST | `/organizations/{id}/journal_entries` | Bearer header |
| Journal lines | POST | `/organizations/{id}/journal_entries/{entry_id}/lines` | Bearer header |
| Journal lines | GET | `/organizations/{id}/journal_entries/{entry_id}/lines` | Bearer header |
| Journal entries | POST | `/organizations/{id}/journal_entries/{entry_id}/post` | Bearer header |
| Accounting | GET | `/organizations/{id}/trial_balance` | Bearer header |
| General ledger | GET | `/organizations/{id}/accounts/{account_id}/ledger` | Bearer header |
| Reports | GET | `/organizations/{id}/reports/profit-loss` | Bearer header |
| Reports | GET | `/organizations/{id}/reports/balance-sheet` | Bearer header |

The route table is the complete set of routes registered in `src/main.rs` in the supplied snapshot.

## 3. Authentication

### Session behavior

`POST /auth/login` creates a session that expires after seven days. The response sets:

```http
Set-Cookie: session=<opaque-token>; HttpOnly; SameSite=Lax; Path=/; Max-Age=604800
```

The session token is stored hashed in the database. For cookie-authenticated routes, send the cookie automatically using a cookie jar:

```bash
curl -i -c cookies.txt \
  -H "Content-Type: application/json" \
  -d '{"email":"you@example.com","password":"your-password"}' \
  "$API/auth/login"
```

Subsequent cookie-authenticated requests use:

```bash
curl -i -b cookies.txt "$API/auth/me"
```

### Authentication compatibility issue

The source has two different authentication extractors:

- `src/services/auth.rs` extracts `session` from the `Cookie` header.
- `src/services/journal_entry_handlers.rs` extracts a `Bearer` token from the `Authorization` header.

The login handler does not return the token in its JSON body; it only sets the HttpOnly cookie. For temporary testing of the bearer-only endpoints, extract the session value from curl's cookie jar:

```bash
TOKEN="$(awk '$6 == "session" { print $7 }' cookies.txt | tail -n 1)"
printf '%s\n' "$TOKEN"
```

Then send:

```bash
-H "Authorization: Bearer $TOKEN"
```

This is a workaround for testing, not a consistent client contract. The recommended implementation fix is to use one shared session extractor for every protected handler, preferably the existing cookie-based approach if the intended clients are browser/TUI clients that already maintain cookies.

## 4. Health

### `GET /health`

No authentication required.

```bash
curl -i "$API/health"
```

Successful response:

```text
HTTP/1.1 200 OK

I am Always Up Master!
```

## 5. Authentication endpoints

### `POST /auth/register`

Creates a user. Registration does not automatically log the user in.

Request:

```json
{
  "email": "you@example.com",
  "password": "a-password-at-least-12-characters"
}
```

Validation in the current handler:
- Email is trimmed and lowercased.
- Email must be non-empty, contain `@`, and contain no whitespace.
- Password must contain at least 12 bytes.

```bash
curl -i -X POST "$API/auth/register" \
  -H "Content-Type: application/json" \
  -d '{"email":"you@example.com","password":"a-password-at-least-12-characters"}'
```

Success: `201 Created`

```json
{"id":"USER_UUID"}
```

Other statuses: `400 Bad Request` for validation failure, `409 Conflict` for a duplicate email, `500 Internal Server Error` for an unexpected database/registration error.

### `POST /auth/login`

Request:

```json
{
  "email": "you@example.com",
  "password": "a-password-at-least-12-characters"
}
```

```bash
curl -i -c cookies.txt -X POST "$API/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"email":"you@example.com","password":"a-password-at-least-12-characters"}'
```

Success: `200 OK`, JSON body:

```json
{"id":"USER_UUID"}
```

Also sets the `session` cookie described above. Invalid credentials return `401 Unauthorized`; empty email/password returns `400 Bad Request`.

### `GET /auth/me`

Requires the `session` cookie.

```bash
curl -i -b cookies.txt "$API/auth/me"
```

Success: `200 OK`

```json
{"id":"USER_UUID"}
```

### `POST /auth/logout`

Requires the `session` cookie. Deletes the current session and expires the cookie.

```bash
curl -i -b cookies.txt -X POST "$API/auth/logout"
```

Success: `204 No Content`.

### `POST /auth/logout-all`

Requires the `session` cookie. Deletes every session for the authenticated user.

```bash
curl -i -b cookies.txt -X POST "$API/auth/logout-all"
```

Success: `204 No Content`.

Implementation note: this handler deletes all sessions but does not send a cookie-expiration header. The browser's cookie may remain stored, although its server-side session is invalid.

### `POST /auth/change-pass`

Requires the `session` cookie.

Request:

```json
{
  "current_password": "current-password",
  "new_password": "a-new-password-at-least-12"
}
```

```bash
curl -i -b cookies.txt -X POST "$API/auth/change-pass" \
  -H "Content-Type: application/json" \
  -d '{"current_password":"current-password","new_password":"a-new-password-at-least-12"}'
```

Success: `204 No Content`. The new password must be at least 12 bytes. The current password must verify. On success, all sessions for the user are deleted, including the session used for the request.

Statuses: `400` for a too-short new password, `401` for missing/invalid session or incorrect current password, `500` for internal failures.

### `POST /auth/forgot-pass`

No authentication required. The handler returns a generic response whether or not the email exists.

Request:

```json
{"email":"you@example.com"}
```

```bash
curl -i -X POST "$API/auth/forgot-pass" \
  -H "Content-Type: application/json" \
  -d '{"email":"you@example.com"}'
```

Success: `202 Accepted`

```json
{
  "message":"If an account exists for that email, reset instructions will be sent."
}
```

For an existing user, the backend creates a reset token with a 15-minute expiry and attempts to send an email. The reset URL is built from `APP_BASE_URL`. SMTP settings are read from environment variables. The handler currently ignores the email-send result, so `202` does not prove that an email was delivered.

### `POST /auth/reset-password`

No authentication required.

Request:

```json
{
  "token": "TOKEN_FROM_RESET_EMAIL",
  "new_password": "a-new-password-at-least-12"
}
```

```bash
curl -i -X POST "$API/auth/reset-password" \
  -H "Content-Type: application/json" \
  -d '{"token":"TOKEN_FROM_RESET_EMAIL","new_password":"a-new-password-at-least-12"}'
```

Success: `204 No Content`. The token must be valid, unused, and unexpired. The new password must be at least 12 bytes. Successful reset deletes all existing sessions for that user.

Statuses: `400` for invalid/expired/used token or invalid password, `500` for internal failures.

## 6. Organizations

All organization endpoints use the `session` cookie.

### `POST /organizations`

Creates an organization and adds the creator as its `owner`.

Request:

```json
{"name":"Example Company"}
```

```bash
curl -i -b cookies.txt -X POST "$API/organizations" \
  -H "Content-Type: application/json" \
  -d '{"name":"Example Company"}'
```

Success: `201 Created`

```json
{"id":"ORGANIZATION_UUID"}
```

Name must be non-empty after trimming and no longer than 120 bytes.

### `GET /organizations`

Lists organizations for the authenticated user.

```bash
curl -i -b cookies.txt "$API/organizations"
```

Success: `200 OK`

```json
{
  "organizations": [
    {
      "id":"ORGANIZATION_UUID",
      "name":"Example Company",
      "role":"owner"
    }
  ]
}
```

### `GET /organizations/{id}`

Returns one organization only if the authenticated user is a member.

```bash
curl -i -b cookies.txt "$API/organizations/ORGANIZATION_UUID"
```

Success: `200 OK`

```json
{
  "id":"ORGANIZATION_UUID",
  "name":"Example Company",
  "role":"owner"
}
```

A non-member or unknown organization returns `404 Not Found`.

### `POST /organizations/{id}/members`

Adds an existing user to an organization. Only a caller whose role is `owner` or `admin` may add members. The requested role must be `admin` or `member`.

Request:

```json
{
  "user_id":"EXISTING_USER_UUID",
  "role":"member"
}
```

```bash
curl -i -b cookies.txt -X POST "$API/organizations/ORGANIZATION_UUID/members" \
  -H "Content-Type: application/json" \
  -d '{"user_id":"EXISTING_USER_UUID","role":"member"}'
```

Success: `201 Created`

```json
{"message":"Member added"}
```

Statuses: `400` for a role other than `admin` or `member`, `401` for missing/invalid session, `403` if caller is not owner/admin, `409` for insertion conflict.

## 7. Accounts

Account endpoints use the `session` cookie. Any organization member can create, list, and retrieve accounts; the current handlers check membership but do not restrict these actions to a particular role.

Valid `account_type` values:
- `asset`
- `liability`
- `equity`
- `revenue`
- `expense`

### `POST /organizations/{id}/accounts`

Request:

```json
{
  "code":"1000",
  "name":"Cash",
  "account_type":"asset",
  "opening_balance":"1500.00"
}
```

```bash
curl -i -b cookies.txt -X POST "$API/organizations/ORGANIZATION_UUID/accounts" \
  -H "Content-Type: application/json" \
  -d '{"code":"1000","name":"Cash","account_type":"asset","opening_balance":"1500.00"}'
```

Success: `201 Created`

```json
{"id":"ACCOUNT_UUID"}
```

Validation: code is non-empty and at most 32 bytes; name is non-empty and at most 120 bytes; account type must be one of the five values above; opening balance must parse as a finite, non-negative number. The current handler accepts `opening_balance` as a JSON **string**.

Duplicate organization/account code and other insertion errors are mapped to `409 Conflict`.

### `GET /organizations/{id}/accounts`

```bash
curl -i -b cookies.txt "$API/organizations/ORGANIZATION_UUID/accounts"
```

Success: `200 OK`

```json
{
  "accounts": [
    {
      "id":"ACCOUNT_UUID",
      "code":"1000",
      "name":"Cash",
      "account_type":"asset",
      "opening_balance":"1500.0000",
      "is_active":true
    }
  ]
}
```

### `GET /organizations/{id}/accounts/{account_id}`

```bash
curl -i -b cookies.txt \
  "$API/organizations/ORGANIZATION_UUID/accounts/ACCOUNT_UUID"
```

Success: `200 OK`, one account object with the same fields as the list response. Returns `404 Not Found` if the account does not exist in that organization.

### Opening balance limitation

The current code stores and returns `opening_balance`. However, the trial-balance query and ledger/report queries calculate from posted journal lines; they do not include the account's `opening_balance` column. Therefore, an account's opening balance is **stored but not yet reflected in those accounting calculations**. Treat this feature as partially implemented.

## 8. Journal entries

**These endpoints currently require `Authorization: Bearer <token>`, unlike the cookie-authenticated endpoints above.** See [Authentication compatibility issue](#authentication-compatibility-issue).

Set the token for the examples after logging in:

```bash
TOKEN="$(awk '$6 == "session" { print $7 }' cookies.txt | tail -n 1)"
export TOKEN
```

Use this header on journal-entry and reporting requests:

```bash
-H "Authorization: Bearer $TOKEN"
```

### `POST /organizations/{id}/journal_entries`

Creates a draft journal entry.

Request:

```json
{
  "entry_date":"2026-11-09",
  "description":"Record office supplies purchase",
  "reference":"INV-001"
}
```

`reference` is optional or may be `null`. Description must be non-empty and at most 500 bytes; reference must be at most 100 bytes.

```bash
curl -i -X POST "$API/organizations/ORGANIZATION_UUID/journal_entries" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"entry_date":"2026-11-09","description":"Record office supplies purchase","reference":"INV-001"}'
```

Success: `201 Created`

```json
{
  "id":"JOURNAL_ENTRY_UUID",
  "entry_date":"2026-11-09",
  "description":"Record office supplies purchase",
  "reference":"INV-001",
  "status":"draft"
}
```

### `POST /organizations/{id}/journal_entries/{entry_id}/lines`

Adds one debit or credit line to a draft entry.

Request:

```json
{
  "account_id":"ACCOUNT_UUID",
  "description":"Cash paid",
  "debit":"0",
  "credit":"250.00"
}
```

Exactly one of `debit` or `credit` must be positive; the other must be zero. Both must be parseable decimal strings and non-negative. Description is optional and at most 500 bytes. The account must belong to the organization and be active. The journal entry must exist and still be a draft.

```bash
curl -i -X POST "$API/organizations/ORGANIZATION_UUID/journal_entries/JOURNAL_ENTRY_UUID/lines" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"account_id":"ACCOUNT_UUID","description":"Cash paid","debit":"0","credit":"250.00"}'
```

Success: `201 Created`

```json
{"id":"JOURNAL_LINE_UUID"}
```

Statuses include `400` for invalid amounts or invalid/inactive account, `401` for missing/invalid bearer token, `403` for non-members, `404` if the entry does not exist, and `409` if the entry is already posted.

### `GET /organizations/{id}/journal_entries/{entry_id}/lines`

Lists the lines for an existing entry.

```bash
curl -i \
  -H "Authorization: Bearer $TOKEN" \
  "$API/organizations/ORGANIZATION_UUID/journal_entries/JOURNAL_ENTRY_UUID/lines"
```

Success: `200 OK`

```json
[
  {
    "id":"JOURNAL_LINE_UUID",
    "account_id":"ACCOUNT_UUID",
    "description":"Cash paid",
    "debit":"0",
    "credit":"250"
  }
]
```

Returns `404 Not Found` if the entry does not exist in the organization.

### `POST /organizations/{id}/journal_entries/{entry_id}/post`

Posts a draft entry.

```bash
curl -i -X POST \
  -H "Authorization: Bearer $TOKEN" \
  "$API/organizations/ORGANIZATION_UUID/journal_entries/JOURNAL_ENTRY_UUID/post"
```

Success: `200 OK` with no response body.

Posting rules enforced by the service:
- Entry must exist and be a draft.
- It must contain at least two lines.
- Total debit and total credit must both be positive.
- Total debit must equal total credit.
- The entry is changed to `posted` and receives `posted_at`.
- The database migrations add protections against modifying posted journal entries/lines.

Returns `404` if the entry is missing and `409 Conflict` if it is not a draft or does not meet the posting rules.

## 9. Accounting queries and reports

These endpoints currently require the bearer header.

### `GET /organizations/{id}/trial_balance`

```bash
curl -i \
  -H "Authorization: Bearer $TOKEN" \
  "$API/organizations/ORGANIZATION_UUID/trial_balance"
```

Success: `200 OK`

```json
{
  "accounts": [
    {
      "id":"ACCOUNT_UUID",
      "code":"1000",
      "name":"Cash",
      "account_type":"asset",
      "is_active":true,
      "total_debit":"500",
      "total_credit":"0"
    }
  ],
  "total_debit":"500",
  "total_credit":"500",
  "balanced":true
}
```

Totals are calculated from journal lines belonging to posted entries. The account opening-balance field is not included in this query.

### `GET /organizations/{id}/accounts/{account_id}/ledger`

Optional query parameters:
- `from`: inclusive start date, `YYYY-MM-DD`
- `until`: inclusive end date, `YYYY-MM-DD`
- `limit`: defaults to 50 and is clamped to 1–100
- `offset`: defaults to 0; negative values are clamped to 0

```bash
curl -i \
  -H "Authorization: Bearer $TOKEN" \
  "$API/organizations/ORGANIZATION_UUID/accounts/ACCOUNT_UUID/ledger?from=2026-01-01&until=2026-11-09&limit=50&offset=0"
```

Success: `200 OK`, an array of ledger rows:

```json
[
  {
    "journal_entry_id":"JOURNAL_ENTRY_UUID",
    "entry_date":"2026-11-09",
    "entry_description":"Record office supplies purchase",
    "reference":"INV-001",
    "journal_line_id":"JOURNAL_LINE_UUID",
    "line_description":"Cash paid",
    "debit":"0",
    "credit":"250",
    "running_balance":"-250"
  }
]
```

Only posted entries are included. If both `from` and `until` are supplied, `from` must not be later than `until`. Returns `404` if the account is not found in the organization.

Implementation note: the running balance window is calculated over the account's posted ledger sequence before the outer date filter is applied.

### `GET /organizations/{id}/reports/profit-loss`

Required query parameters: `from` and `until`, inclusive, as `YYYY-MM-DD`.

```bash
curl -i \
  -H "Authorization: Bearer $TOKEN" \
  "$API/organizations/ORGANIZATION_UUID/reports/profit-loss?from=2026-01-01&until=2026-11-09"
```

Success: `200 OK`

```json
{
  "revenue": [
    {
      "account_type":"revenue",
      "account_id":"ACCOUNT_UUID",
      "account_name":"Sales",
      "amount":"1000"
    }
  ],
  "expenses": [
    {
      "account_type":"expense",
      "account_id":"ACCOUNT_UUID",
      "account_name":"Rent",
      "amount":"250"
    }
  ],
  "total_revenue":"1000",
  "total_expenses":"250",
  "net_profit":"750"
}
```

Returns `400 Bad Request` if `from` is later than `until`.

### `GET /organizations/{id}/reports/balance-sheet`

Required query parameter: `as_of`, as `YYYY-MM-DD`.

```bash
curl -i \
  -H "Authorization: Bearer $TOKEN" \
  "$API/organizations/ORGANIZATION_UUID/reports/balance-sheet?as_of=2026-11-09"
```

Success: `200 OK`

```json
{
  "as_of":"2026-11-09",
  "assets": [
    {
      "account_id":"ACCOUNT_UUID",
      "account_name":"Cash",
      "amount":"500"
    }
  ],
  "liabilities": [],
  "equity": [],
  "total_assets":"500",
  "total_liabilities":"0",
  "total_equity":"0",
  "current_earnings":"500",
  "balanced":true
}
```

The report calculates balances from posted journal entries up to `as_of`. `balanced` indicates whether total assets equal total liabilities plus total equity plus current earnings. Opening balances are not included by the current SQL queries.

## 10. Status codes and error format

The handlers generally return an HTTP status code without a structured JSON error body. Do not assume an error response contains `{ "error": ... }`.

Common statuses:
- `200 OK`: successful read or action.
- `201 Created`: resource created.
- `202 Accepted`: password reset request accepted for processing.
- `204 No Content`: successful action with no response body.
- `400 Bad Request`: invalid request data, query parameters, or amount.
- `401 Unauthorized`: missing/invalid session or bearer token, or invalid credentials.
- `403 Forbidden`: authenticated user is not permitted to access the organization.
- `404 Not Found`: requested organization/resource not found (depending on endpoint).
- `409 Conflict`: duplicate/conflicting resource or invalid state transition.
- `429 Too Many Requests`: rate limit exceeded on endpoints with a Governor layer.
- `500 Internal Server Error`: unexpected server/database error.

Some report handlers currently map internal query/decimal errors to `418 I'm a Teapot`. This is an implementation detail and should be normalized to `500 Internal Server Error` before a stable public API release.

## 11. Rate limiting, CORS, and request limits

From `src/main.rs`:
- Request body limit: 64 KiB.
- CORS allows origins `http://localhost:5173` and `http://127.0.0.1:5173`.
- CORS allows credentials.
- Allowed methods: GET, POST, OPTIONS.
- Allowed headers: `Content-Type`, `Authorization`.
- Rate limiting is attached to register, login, forgot-password, and reset-password routes. The current governor configurations use peer IP extraction and the configured per-second/burst values in `src/main.rs`.

## 12. Known gaps and follow-up work

These are absent from the registered route set in the supplied source snapshot, or are visibly incomplete in the current implementation:

| Feature | Current status | Notes |
|---|---|---|
| Consistent authentication across API | **Needs fix** | Cookie auth in auth/org/account handlers; bearer auth in journal/report handlers. |
| Opening balance accounting integration | **Partial** | Stored and returned on accounts, but excluded from trial balance, general ledger, P&L, and balance sheet calculations. |
| List journal entries | **Not exposed** | No `GET /organizations/{id}/journal_entries` route. |
| Get one journal entry | **Not exposed** | No individual journal-entry GET route. |
| Update/delete draft journal entry | **Not exposed** | No update or delete routes. |
| Update/deactivate account | **Not exposed** | No account update or deactivate route. |
| Delete account | **Not exposed** | No account delete route. |
| List organization members | **Not exposed** | Add-member exists, but no members-list route. |
| Remove member/change member role | **Not exposed** | No routes registered for these operations. |
| Structured API error responses | **Not implemented** | Errors are generally bare HTTP statuses. |
| Consistent internal-error status mapping | **Needs cleanup** | Some report paths return 418 for internal failures. |
| Password reset email delivery confirmation | **Partial** | Email send result is ignored by the forgot-password handler. |
| Automated OpenAPI/Swagger schema | **Not present** | No OpenAPI route or schema generation is registered. |

## 13. Local Docker setup

From the repository root:

```bash
docker compose up -d
docker compose ps
docker compose logs -f backend
```

The API is exposed on port `3000` by the Compose configuration. The backend runs migrations at startup.

Health check:

```bash
curl -i http://localhost:3000/health
```

## 14. Suggested API verification order

For a clean manual smoke test:

1. `GET /health`
2. `POST /auth/register`
3. `POST /auth/login` (save cookie jar)
4. `GET /auth/me`
5. `POST /organizations`
6. `GET /organizations`
7. `POST /organizations/{id}/accounts`
8. `GET /organizations/{id}/accounts`
9. Create a journal entry and add at least two balanced lines.
10. Post the journal entry.
11. Query trial balance, ledger, P&L, and balance sheet.

**Before publishing this API as stable:** resolve the authentication mismatch and opening-balance accounting gap, then run the smoke-test sequence against the Docker deployment.
