# SUMILEDGER

You might be wondering, what is **SumiLedger**?

SumiLedger is an accounting software, yea... just like Tally, etc.

SumiLedger aims to provide a simple and reliable accounting experience, with the flexibility of a modern software architecture.

This iteration of SumiLedger currently includes:

- **Accounts** – Manage accounting accounts.
- **Journal** – Record and manage journal entries.
- **Balance Sheet** – View the financial position of your business.

More features are planned for future iterations!

---

## Deployment

SumiLedger provides Docker images for its backend and CLI (Sumi TUI).

### Docker Images

**Backend**
```bash
docker pull yumeer/sumiledger-backend:tagname
```

**CLI (Sumi TUI)**
```bash
docker pull yumeer/sumiledger-cli:tagname
```

Replace `tagname` with the version you want to use.

### Publishing Docker Images

If you have built the images locally and want to publish them to Docker Hub:

**Backend**
```bash
docker push yumeer/sumiledger-backend:tagname
```

**CLI (Sumi TUI)**
```bash
docker push yumeer/sumiledger-cli:tagname
```

Make sure you're logged in to Docker Hub before pushing:

```bash
docker login
```

---

## Project Components

| Component | Description |
|---|---|
| SumiLedger Backend | Handles backend operations and accounting logic |
| Sumi TUI | Terminal-based user interface |

---

## Roadmap

- [x] Accounts
- [x] Journal
- [x] Balance Sheet
- [ ] More accounting reports
- [ ] Additional accounting features

---

## Contributing

Contributions, suggestions, and bug reports are welcome!

Feel free to open an issue or submit a pull request.

---

**SumiLedger** – Accounting, simplified.