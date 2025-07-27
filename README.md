# 🛡️ Sentiric SIP Gateway Service

**Description:** This service acts as a hardened **Session Border Controller (SBC)** and intelligent **load balancer** for the Sentiric platform. It is designed to be the first point of contact for all external SIP traffic, providing security, protocol normalization, and high availability.

**Strategic Vision:**
While industry-standard tools like Kamailio or OpenSIPS offer extensive features, our philosophy is to build a lightweight, purpose-built gateway that perfectly fits our microservice architecture without unnecessary complexity or external dependencies. We are not rebuilding a full-featured PBX; we are crafting a specialized, high-performance security and routing layer.

**Core Responsibilities (Future):**
*   **Security Enforcement:** Acts as the primary line of defense against DDoS attacks, SIP spam/flooding, and scanners. It will perform IP filtering, rate limiting, and sanity checks on incoming SIP packets before forwarding them to the core `sip-signaling-service`.
*   **High Availability & Load Balancing:** When multiple `sip-signaling-service` instances are running, this gateway will distribute incoming calls among them, ensuring service continuity even if one instance fails.
*   **Protocol Normalization:** Will clean up or modify SIP headers from non-standard external providers to ensure the internal `sip-signaling-service` only deals with clean, predictable data.
*   **WebRTC to SIP Bridging:** Will act as a bridge to convert WebRTC traffic from browsers into the standard SIP protocol used internally.

**Current Status:**
This service is part of the **long-term vision** for the Sentiric platform. In the current phase, the `sentiric-sip-signaling-service` directly handles external traffic for simplicity and rapid development. The `sip-gateway-service` will be implemented as the platform scales and faces enterprise-level security and availability requirements.

**Technology:**
*   **Language:** Rust (for performance, security, and low-level network control).