## Finanzguru für Firefly III mit AI, Forecasting, Grafana und Crypto-Portfolio

---

# Vision

Flow Finance AI ist ein modernes, vollständig selbst gehostetes Finanzanalyse-, Planungs- und Forecasting-System.

Es erweitert Firefly III um:

- Subscription-Erkennung
    
- Dauerauftrags-Erkennung
    
- Cashflow-Prognosen
    
- Finanzplanung
    
- Szenario-Simulationen
    
- Portfolio-Management
    
- Crypto-Integration
    
- AI-Assistent
    
- Grafana Analytics
    
- Frühwarnsysteme
    
- Vermögensentwicklung
    
- Plan-vs-Ist-Vergleiche
    

Ziel ist ein System ähnlich Finanzguru, jedoch vollständig selfhosted, erweiterbar und datenschutzfreundlich.

---

# Grundprinzip

Firefly III bleibt die einzige Quelle für Finanztransaktionen.

```text
Banken
Kreditkarten
CSV
GoCardless
PSD2
      ↓
Firefly III
      ↓
Flow Finance AI
      ↓
Analyse
Forecasting
Planung
AI
Grafana
```

Das System verändert keine Firefly-Daten.

Es liest ausschließlich Daten aus Firefly III und erzeugt eigene Analysen, Prognosen und Planungen.

---

# Architektur

```text
┌─────────────────────┐
│ Firefly III         │
└──────────┬──────────┘
           │ API
           ▼
┌─────────────────────┐
│ Flow Finance AI     │
│ Rust Backend        │
└───────┬─────────────┘
        │
        ▼
┌─────────────────────┐
│ PostgreSQL          │
│ TimescaleDB         │
└───────┬─────────────┘
        │
        ├──────────────┐
        ▼              ▼
┌──────────────┐ ┌──────────────┐
│ Grafana      │ │ React UI     │
└──────────────┘ └──────────────┘
                       │
                       ▼
                ┌──────────────┐
                │ AI Assistant │
                └──────────────┘
```

---

# Technologie-Stack

## Backend

|Bereich|Technologie|
|---|---|
|Sprache|Rust|
|Web Framework|Axum|
|Async Runtime|Tokio|
|Datenbank|PostgreSQL|
|Zeitreihen|TimescaleDB|
|ORM/DB|SQLx|
|Scheduler|Tokio Cron Scheduler|
|HTTP Client|Reqwest|
|Websocket|Tokio Tungstenite|
|Config|Serde + Config|
|Logging|Tracing|

---

## Frontend

|Bereich|Technologie|
|---|---|
|Framework|React|
|Sprache|TypeScript|
|Styling|TailwindCSS|
|Komponenten|shadcn/ui|
|State|TanStack Query|
|Charts|Apache ECharts|
|Auth|OIDC|

---

## Analytics

|Bereich|Technologie|
|---|---|
|Dashboard|Grafana|
|Alerts|Grafana Alerting|
|Reports|Eigene React UI|

---

## AI

|Bereich|Technologie|
|---|---|
|Start|OpenAI|
|Zukunft|Ollama|
|Zukunft|LM Studio|
|Zukunft|OpenAI Compatible APIs|
|Zukunft|LocalAI|
|Zukunft|vLLM|

---

# Docker Architektur

## Minimal

```text
flow-finance-ai
firefly-iii
grafana
```

---

## Standard

```text
flow-finance-ai
firefly-iii
grafana
redis
```

---

## Vollständig

```text
flow-finance-ai
firefly-iii
grafana
redis
ollama
```

---

# Konfiguration

PostgreSQL darf niemals fest eingebaut sein.

```toml
[database]
mode = "external"

host = "postgres"
port = 5432

database = "flow_finance_ai"
user = "finance"
password = "secret"

ssl = false
```

---

# Module

## Firefly Connector

Verantwortlich für:

- API Verbindung
    
- Datenimport
    
- Synchronisierung
    
- Caching
    

Importiert:

- Konten
    
- Transaktionen
    
- Kategorien
    
- Budgets
    
- Tags
    
- Piggy Banks
    

---

# Subscription Detection Engine

Erkennt automatisch:

```text
Netflix
Spotify
Amazon
Handy
Versicherung
Leasing
Strom
Gas
```

Analyse anhand von:

- Betrag
    
- Empfänger
    
- Text
    
- Regelmäßigkeit
    
- Intervallen
    

---

## Confidence Score

```text
95% = sehr sicher
80% = wahrscheinlich
60% = prüfen
```

---

## Benutzerbestätigung

```text
Neue Subscription erkannt

Netflix
Monatlich
15,99 €

[Bestätigen]
[Ablehnen]
```

---

# Preisänderungs-Erkennung

Beispiel:

```text
Spotify

12,99 €
13,99 €
15,99 €
```

Erkennung:

- Preiserhöhungen
    
- Preissenkungen
    
- Häufigkeit
    

---

# Forecast Engine

Berechnet:

## Tägliche Prognosen

```text
Kontostand morgen
Kontostand nächste Woche
Kontostand Monatsende
```

---

## Monatliche Prognosen

```text
Einnahmen
Fixkosten
Variable Kosten
Freier Cashflow
```

---

## Langfristige Prognosen

```text
3 Monate
6 Monate
12 Monate
24 Monate
```

---

# Plan Engine

Erlaubt verschiedene Szenarien.

## Beispiele

### Aktuell

```text
Ist-Zustand
```

### Leasing Auto

```text
+300 €/Monat
```

### Sparmodus

```text
Netflix weg
Spotify weg
Essen -100 €
```

### Hauskauf

```text
Sparrate erhöhen
```

---

# Plan Versionierung

Jeder Plan besitzt Versionen.

```text
Plan Leasing

v1
v2
v3
```

Vergleich möglich.

---

# Plan-vs-Ist

Täglicher Vergleich.

```text
Geplant:

1.200 €

Tatsächlich:

1.050 €

Abweichung:

-150 €
```

---

# Alert Engine

## Warnungen

### Knappheit

```text
Kontostand fällt unter 200 €
```

---

### Subscription Änderung

```text
Netflix wurde teurer
```

---

### Neues Abo

```text
Neue regelmäßige Zahlung erkannt
```

---

### Budget Drift

```text
Lebensmittel +20 %
```

---

### Plan Gefährdet

```text
Leasing Plan nicht tragfähig
```

---

# Vermögensanalyse

Berechnet:

```text
Girokonto
Sparkonto
Tagesgeld
Firefly Konten
Crypto
```

=

```text
Gesamtvermögen
```

---

# Crypto Erweiterung (Phase 2)

## Ziel

Crypto vollständig ins Vermögen integrieren.

---

## Unterstützte Börsen

### Start

- Binance
    
- Bybit
    
- Bitunix
    

---

### Zukunft

- Kraken
    
- Coinbase
    
- Bitpanda
    
- OKX
    

---

# Crypto Daten

Importiert:

```text
Wallets
Balances
Positionen
Trades
Transfers
Funding
PnL
```

---

# Portfolio Engine

Berechnet:

## Realisierte Gewinne

```text
Verkaufte Positionen
```

---

## Nicht realisierte Gewinne

```text
Offene Positionen
```

---

## Gesamtrendite

```text
Portfolio Performance
```

---

# Portfolio Planung

Beispiele:

```text
Monatlich 300 € investieren
```

```text
50 % ETF
50 % Crypto
```

```text
70 % ETF
20 % Crypto
10 % Cash
```

---

# AI Assistant

## Ziele

Mit den Finanzdaten sprechen können.

---

## Beispiele

```text
Kann ich mir ein Leasing Auto leisten?
```

```text
Welche Abos wurden teurer?
```

```text
Warum bin ich diesen Monat über Budget?
```

```text
Wie viel spare ich wenn ich Netflix kündige?
```

```text
Welche Kategorien kosten am meisten?
```

---

# AI Tool Layer

AI darf niemals direkt auf Datenbanken zugreifen.

Nur über Tools.

```text
get_transactions()

get_subscriptions()

get_forecast()

get_budget_status()

get_portfolio()

simulate_plan()
```

---

# Datenschutz

## Privacy Layer

Optionen:

```toml
allow_raw_transactions = false

redact_iban = true

redact_counterparties = true
```

---

# Grafana Dashboards

## Dashboard 1

Cashflow

```text
Kontostand
Forecast
Knappheiten
```

---

## Dashboard 2

Subscriptions

```text
Alle Abos
Preisänderungen
Neue Abos
```

---

## Dashboard 3

Budgets

```text
Plan
Ist
Abweichung
```

---

## Dashboard 4

Portfolio

```text
Gesamtvermögen
Crypto
Performance
```

---

## Dashboard 5

Forecast

```text
1 Monat
3 Monate
6 Monate
12 Monate
```

---

# Roadmap

## Phase 1

MVP

- Firefly Integration
    
- PostgreSQL
    
- React UI
    
- Grafana
    
- Forecasting
    

---

## Phase 2

Subscriptions

- Subscription Detection
    
- Preisänderungen
    
- Alerts
    

---

## Phase 3

Planung

- Szenarien
    
- Plan-vs-Ist
    
- Versionierung
    

---

## Phase 4

AI

- OpenAI
    
- Tool Calling
    
- Chat
    

---

## Phase 5

Crypto

- Binance
    
- Bybit
    
- Bitunix
    

---

## Phase 6

Local AI

- Ollama
    
- LM Studio
    
- OpenAI Compatible APIs
    

---

## Phase 7

Advanced Forecasting

- Machine Learning
    
- Saisonale Modelle
    
- Portfolio Prognosen
    
- Risikobewertungen
    

---

# Langfristige Vision

Ein vollständig selbst gehostetes Finanzbetriebssystem, das:

- alle Konten kennt
    
- alle Börsen kennt
    
- alle Abos kennt
    
- Vermögen analysiert
    
- Risiken erkennt
    
- Sparpotenziale findet
    
- Finanzpläne simuliert
    
- AI-gestützt Empfehlungen gibt
    
- vollständig unter eigener Kontrolle läuft
    

ohne Abhängigkeit von Finanzguru, Banken oder Cloud-Anbietern.



# zusatzinfkl zu Infra zur entwicklungszeit:
Docker ist  hier installiert, FIrefly III löuft hier
Verbiondung zur APi kann direkt getestet und eingerichtet werden innerhalb dieser Umgebung
Parameter über Docker Compose zu nehmen!