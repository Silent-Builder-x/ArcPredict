# ArcPredict: FHE-Powered Anti-Manipulation Prediction Market

## 🔮 Overview

**ArcPredict** is a next-generation decentralized prediction market built on **Arcium** and **Solana**.

Current prediction markets (like Polymarket) suffer from "Herding Bias" and price manipulation because every bet and its direction are visible in real-time. **ArcPredict** leverages **Fully Homomorphic Encryption (FHE)** to hide individual positions and the current state of the pool. The market remains a "Dark Pool" until the event is resolved, ensuring that participants vote based on their true beliefs rather than following the crowd.

## 🚀 Live Deployment Status (Verified)

The protocol is fully functional and currently active on the Arcium Devnet.

- **MXE Address:** `21mvYC5anZRsA3zsWm8Gn5M4Gtp8FSGrgnRxj42umVJ4`
- **MXE Program ID:** `5bzLvR4rs6gAPQHfWTMmX3n6xXe6gPPM8iV5mAGxcoYe`
- **Computation Definition:** `3zp7E5Sd67Wpjcy18em4KHPBBXb2kPQWdzaTtXYs3Urc`
- **Authority:** `AjUstj3Qg296mz6DFcXAg186zRvNKuFfjB7JK2Z6vS7R`
- **Status:** `Active`

## 🧠 Core Innovation: The "Odds Nebula"

ArcPredict implements a **Confidential Aggregation Engine** using Arcis FHE circuits:

- **Shielded Stakes:** Both the amount and the choice (YES/NO) are encrypted locally using x25519 before being sent on-chain.
- **Homomorphic Pool Updates:** The Arcium MXE nodes update the pool weights in their encrypted state using secure multiplexers (`if-else` mux).
- **Privacy-First Settlement:** The winning payout ratio is calculated homomorphically: $Payout = \frac{TotalPool}{WinningPool} \times UserBet$. Only the final result is decrypted for the winner.

## 🛠 Build & Implementation

```
# Compile Arcis circuits and Anchor program
arcium build

# Deploy to Cluster 456
arcium deploy --cluster-offset 456 --recovery-set-size 4 --keypair-path ~/.config/solana/id.json -u d

```

## 📄 Technical Specification

- **Core Logic:** `update_market_state` & `resolve_payout` (Arcis-FHE)
- **Security:** Multi-party threshold signatures with a recovery set size of 4.
- **Audit Compliance:** Strict Anchor safety standards with verified `/// CHECK:` documentation.