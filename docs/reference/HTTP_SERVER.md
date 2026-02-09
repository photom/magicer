# HTTP Server Specification - Linux File Magic API <!-- omit in toc -->

- [1. Concurrency and Queue Management](#1-concurrency-and-queue-management)
- [2. Timeout Specifications](#2-timeout-specifications)
- [3. Resource and Payload Constraints](#3-resource-and-payload-constraints)
- [4. File System Sandbox Specification](#4-file-system-sandbox-specification)
- [5. Operational Specifications](#5-operational-specifications)
  - [Sources for Technical Standards](#sources-for-technical-standards)


This document defines the technical specifications for the HTTP server layer. This is a formal specification of the server's operational limits and behavior.

## 1. Concurrency and Queue Management

The server manages incoming traffic using a combination of active connection limits and a request backlog at the operating system level.

| Parameter | Value | Specification |
| --- | --- | --- |
| **Max Concurrent Connections** | 1,000 | The maximum number of active TCP connections handled by the application layer simultaneously. |
| **Backlog Size** | 1,024 | The maximum number of pending connections in the TCP listen queue (OS level). Connections exceeding this will be refused. |

## 2. Timeout Specifications

| Timeout Type | Value | Specification |
| --- | --- | --- |
| **Request Read** | 60s | The maximum window for receiving the complete HTTP request (header and body). |
| **Response Write** | 60s | The maximum window for transmitting the full response body to the client. |
| **Analysis Execution** | 30s | The maximum time allocated for the internal file magic identification process. |
| **Keep-Alive** | 75s | The duration an idle connection is maintained before forced closure. |

## 3. Resource and Payload Constraints

| Constraint | Value | Specification |
| --- | --- | --- |
| **Max Request Body** | 100MB | The hard limit for the `application/octet-stream` payload. |
| **Max URI Length** | 8KB | Limit for the URI string including all query parameters. |
| **Max Header Size** | 16KB | Total cumulative size allowed for all HTTP request headers. |

## 4. File System Sandbox Specification

The server enforces a strict security boundary for all file-path-based operations:

* **Root Isolation:** The server operates relative to a pre-defined base directory.
* **Path Resolution:** All incoming path strings are sanitized. References to `..` (parent), `.` (current), or leading `/` (root) are prohibited and result in immediate `403 Forbidden` or `400 Bad Request`.
* **Symbolic Link Policy:** The server is specified to ignore or reject symbolic links that resolve to a location outside the base directory.

## 5. Operational Specifications

* **Signal Handling:** The server intercepts `SIGTERM` and `SIGINT` to perform a graceful shutdown, allowing a 10-second window for in-flight requests to finalize before process termination.
* **Request Correlation:** Every log entry and response body is bound by a unique Request ID (UUID v4) for cross-system tracing.
* **Response Content-Type:** All responses, including error messages, are delivered strictly as `application/json` with UTF-8 encoding.
* **Health State:** The `/v1/ping` endpoint specifies the server's availability. A `200 OK` response indicates the internal task executors and the HTTP listener are functional.

---

### Sources for Technical Standards

* **TCP Backlog:** [Linux Listen(2) Man Page](https://man7.org/linux/man-pages/man2/listen.2.html)
* **HTTP Semantics:** [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html)
* **File Security:** [OWASP File Storage Security Guide](https://www.google.com/search?q=https://cheatsheetseries.owasp.org/cheatsheets/File_Storage_Security_Cheat_Sheet.html)
