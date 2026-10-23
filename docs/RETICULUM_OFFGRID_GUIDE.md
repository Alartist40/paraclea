# Paraclea Reticulum Off-Grid Mesh Networking Guide

This guide details how Paraclea integrates with the **Reticulum Network Stack (RNS)** to provide sovereign, encrypted, off-grid communications across local peers without cellular coverage, internet access, or central servers.

---

## 1. What is Reticulum?

Reticulum is a cryptography-based networking stack built for resilient communications:
- **No IP Addresses**: Nodes do not have IP addresses. Destinations are 128-bit truncated SHA-256 hashes of cryptographic public keys.
- **End-to-End Encryption by Default**: All communications are encrypted with Curve25519 (ECDH) and authenticated with Ed25519 signatures.
- **Medium Agnostic**: Operates over LoRa radio transceivers, packet radio, ad-hoc Wi-Fi, serial ports, or Ethernet cables.
- **Autonomous Multi-Hop Routing**: Packets discover the shortest path through available nodes automatically.

---

## 2. Paraclea's Mesh Architecture

```
                                  [Paraclea Node A]
                               (Identity: <hash_a>)
                                         │
                                [ReticulumEngine]
                              (crates/core/mesh.rs)
                                         │
                     ┌───────────────────┴───────────────────┐
                     v                                       v
         [Auto-Announce Beacon]                    [Encrypted Mailbox]
         (Every 15 min broadcast)               (~/.paraclea/mesh/mailbox)
                     │                                       │
                     └───────────────────┬───────────────────┘
                                         │
                             [Physical Medium: LoRa / Wi-Fi]
                                         │
                     ┌───────────────────┴───────────────────┐
                     v                                       v
             [Intermediate Node]                     [Paraclea Node B]
            (Store & Forward Cache)                (Identity: <hash_b>)
```

---

## 3. How to Set Up an Off-Grid Paraclea Mesh

### Step 1: Install Reticulum Tools
On any Linux device (Raspberry Pi, laptop, Orange Pi):
```bash
pip install rns
```

Verify the tools are installed:
```bash
rnstatus
```

### Step 2: Configure a Physical Radio Interface (Optional)
To communicate over long distances (5–30 km) without Wi-Fi, plug in a USB LoRa dongle (e.g. Heltec V3, LilyGO T-Beam, or RNode SX1262):
Edit `~/.reticulum/config`:
```ini
[[LoRa Interface]]
  type = RNodeInterface
  interface_enabled = True
  port = /dev/ttyUSB0
  frequency = 868000000 # or 915000000 for Americas
  bandwidth = 125000
  txpower = 17
  spreadingfactor = 8
  codingrate = 5
```

### Step 3: Launch Paraclea Mesh
When you start Paraclea (`paraclea`):
1. `ReticulumEngine::new()` automatically ensures `~/.paraclea/mesh/identity` is generated.
2. It starts or connects to the local `rnsd` daemon.
3. Typing `/mesh` opens the live Mesh Deck:
   - Displays your unique hexadecimal Cryptographic Destination.
   - Shows all discovered nearby peers and their hop count.
   - Allows sending and reading encrypted peer notes and emergency alerts.

---

## 4. Practical Off-Grid Use Cases for Paraclea

1. **Remote Wilderness Study Groups**: Multiple people in remote areas sharing Scripture highlights and Dendrite notes without cellular service.
2. **Emergency Triage Broadcasts**: Transmitting field medical advice or distress beacons across a localized community mesh during grid outages.
3. **Decentralized Library Syncing**: Synchronizing cross-references and study bookmarks as peers physically come within radio range.
