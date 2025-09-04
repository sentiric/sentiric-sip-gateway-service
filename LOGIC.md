```mermaid
sequenceDiagram
    participant Telekom
    participant SipGateway
    participant SipSignaling
    Telekom->>SipGateway: INVITE
    SipGateway->>SipSignaling: INVITE (başlıkları değiştirilmiş)
```