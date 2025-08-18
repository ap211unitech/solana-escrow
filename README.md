# 🔄 Solana Escrow App

The **Escrow App** is a Solana smart contract built using [Anchor](https://www.anchor-lang.com/) that allows users to securely exchange SPL tokens in a trustless way.  
Each offer is backed by a **dedicated vault account**, ensuring tokens are locked until the offer is either taken by another user or canceled by the maker, ensuring fairness and eliminating counterparty risk.

---

## 📜 Overview

The program supports:

- **Make Offer**:  
  A user (maker) creates an offer by locking a specified amount of `Token A` in a vault, while specifying how much `Token B` they expect in return.

- **Take Offer**:  
  Another user (taker) accepts an existing offer by sending the required `Token B` amount to the maker. In return, the taker receives the locked `Token A` from the vault.

- **Cancel Offer**:  
  The maker can cancel their offer before it's taken, retrieving their locked tokens from the vault and closing appropriate accounts.

## ⚙️ Program Structure

```plaintext
src/
 ├─ instructions/   # Business logic for each instruction
 ├─ states/         # Account data structures for offer
 ├─ constants.rs
 ├─ utils.rs
 └─ lib.rs          # Program entry point
```

## 🛠️ Building & Deploying

### 1. Install Prerequisites

- Solana CLI
- Anchor CLI
- Rust (latest stable)
- Quick setup (https://solana.com/docs/intro/installation)

### 2. Clone Repository

```
git clone https://github.com/ap211unitech/solana-escrow.git
cd solana-escrow
```

### 3. Build Program

```
anchor build
```

### 4. Running Tests

```
solana-test-validator (keep it running in a seperate terminal)
```

and

```
anchor test --skip-local-validator
```

## 📄 License

MIT License © 2025 Arjun Porwal
