---
name: logging-principles
description: Guidelines for structured logging, telemetry, data privacy, and log levels. Use to ensure consistent observability.
---

## 1. Data Privacy & Masking

- **PII Protection**: Never log plain-text passwords, phone numbers, or highly sensitive personal data.
- **Masking Rules**: 
  - Mask values for keys ending in `password`.
  - Mask values for keys starting with `sec_`.
  - Use common logging utility methods or filters to replace sensitive data with masks (e.g., `***`).
- **Array Handling**: Avoid logging indeterminate or large arrays in a single line. If the count exceeds human-readable limits, omit it from the logs.

## 2. Modern Observability & Telemetry

- **Structured Logging**: Use JSON or other structured formats (e.g., key-value pairs) to facilitate integration with Cloud services and SIEM (Security Information and Event Management) platforms.
- **Audit & Security**: Ensure audit/access logs comply with security requirements, including tampering detection and long-term retention.
- **Unified Telemetry**: Integrate Logs, Metrics, and Traces as a unified telemetry strategy to achieve high observability and real-time system insights.

## 🖥️ Server-Side Logging Guidelines

### Operational Context

Logs must empower operators.

**Required Context for Milters:**
Always include these on the same line if available:
- `Connection ID` / `Session ID`
- `Client IP` / `Hostname`

**Metrics (KGI/KPI/KAI)**: Output logs related to defined business and activity indicators (e.g., messages processed, spam detected, virus detected, quarantine actions) to enable continuous improvement cycles.

### Log Levels & Timing

| Level | Timing / Stage | Required Context |
| :--- | :--- | :--- |
| **INFO** | Milter Start/End, Connection Accepted, External API Calls | Connection ID, Client IP, Message-ID, Action taken (Accept, Reject, Discard), External request status. |
| **WARN** | External API timeouts (non-fatal), Malformed headers | Connection ID, Error details, Endpoint/Params for external calls, Specific malformed data (non-sensitive). |
| **ERROR** | Unhandled exceptions, Database connection failures, Critical external service failures causing message tempfails (`4xx`) | Connection ID, Message-ID (if known), Error content, Stack trace (on a separate line). |
| **DEBUG** | Detailed parsing of headers/body, raw bytes received, internal state transitions | Connection ID, Class/Method names, specific header keys being checked, return values of internal functions. |

## 💻 Client-Side Logging Guidelines

*(Included for completeness if building client admin interfaces for the milter)*

**Debugging & Support**
For on-premise or client applications, ensure logs provide necessary information for IT administrators and support teams.

| Level | Timing | Required Context |
| :--- | :--- | :--- |
| **DEBUG** | Public Method Entry/Exit | Resource ID, Class/Method names, Arguments, Return values. |
| **INFO** | App Start/End, Validation Success, Successful mutations, External Comms | Version, Args, Input values, Status codes, Request/Response bodies. |
| **WARN** | External 5xx errors (Permanent/Input errors), Config validation failure | Resource ID, Endpoint, Error line/item, Failed input values. |
| **ERROR** | Unhandled exceptions, External 4xx errors (Temporary/System errors), App crash | Resource ID, Error content, Stack trace (on a separate line). |

## Related Skills

- For Rust implementation specifics, refer to the **rust-master** skill.
- For ensuring third-party interactions are properly logged and masked, see **external-integration**.