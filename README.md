# Sentiric SIP Gateway Service

**Description:** Acts as an intermediary between Sentiric's internal SIP network and external SIP networks or service providers (SIP Trunking).

**Core Responsibilities:**
*   Routing incoming SIP calls from external SIP trunks to `sentiric-sip-server`.
*   Routing outgoing calls from `sentiric-sip-server` to external SIP trunks.
*   Managing specific SIP headers, CODEC negotiations, and NAT traversal for external providers.
*   Acting as a first barrier for security (IP filtering, rate limiting) and fraud detection for external SIP traffic.

**Technologies:**
*   Node.js (or Go)
*   UDP/TCP/TLS Sockets for SIP communication

**API Interactions (As a Protocol Bridge):**
*   Communicates directly with `sentiric-sip-server` via SIP protocol.
*   Interacts with external SIP providers/trunks.

**Local Development:**
1.  Clone this repository: `git clone https://github.com/sentiric/sentiric-sip-gateway-service.git`
2.  Navigate into the directory: `cd sentiric-sip-gateway-service`
3.  Install dependencies: `npm install` (Node.js) or `go mod tidy` (Go).
4.  Create a `.env` file from `.env.example` to configure SIP listening ports and external trunk details.
5.  Start the service: `npm start` (Node.js) or `go run main.go` (Go).

**Configuration:**
Refer to `config/` directory and `.env.example` for service-specific configurations, including SIP trunk details, IP filtering rules, and rate limits.

**Deployment:**
Designed for containerized deployment (e.g., Docker, Kubernetes). Refer to `sentiric-infrastructure`.

**Contributing:**
We welcome contributions! Please refer to the [Sentiric Governance](https://github.com/sentiric/sentiric-governance) repository for coding standards and contribution guidelines.

**License:**
This project is licensed under the [License](LICENSE).
