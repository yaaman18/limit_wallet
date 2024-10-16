# Limit Wallet - Bitcoin Limit Order Wallet

## Motivation
The primary goal of Limit Wallet is to increase the liquidity of Bitcoin. Bitcoin is a deflationary asset, and many users tend to HODL (hold) it as a store of value. This wallet introduces the concept of **"time"** as added value, providing a mechanism that incentivizes users to engage in transactions (value exchange).

Specifically, buyers "purchase" **time** by setting a target Bitcoin rate, waiting until the price reaches that rate. Once the target rate is achieved, the transaction is automatically executed. This enables Bitcoin holders to exchange value at the right moment, improving Bitcoin's liquidity.

Additionally, sellers benefit from the possibility that the value of Bitcoin will increase before the transaction is finalized. While they "lose time," they may gain **increased value** by receiving Bitcoin that has appreciated during the waiting period. This system creates an incentive to not just HODL Bitcoin, but to use it more efficiently in transactions.

## Overview
Limit Wallet syncs with the Bitcoin Testnet network, allowing users to check their Bitcoin balance, create, sign, and broadcast transactions. The wallet is built using Tauri, BDK (Bitcoin Dev Kit), and Svelte.

## Features
- **Sync with the Bitcoin Testnet**: Connect and sync your wallet with the Testnet to check balances and update in real time.
- **Create and sign PSBT (Partially Signed Bitcoin Transactions)**: Support for creating, signing, saving, and broadcasting PSBT transactions.
- **Fetch Bitcoin price**: Retrieve the current Bitcoin price from an API every minute.
- **Automatic wallet synchronization**: The front end is automatically notified when the balance changes.

## Key Functions
1. **Wallet creation and initialization**: The wallet is created by obtaining a mnemonic from environment variables and generating an extended private key (XPRV).

2. **Sync with the Testnet**: Sync the wallet with the Testnet using an Electrum server. The synchronization runs in the background, and the front end is notified when it completes.

3. **Fetch Bitcoin price**: The current Bitcoin price is fetched every minute via an API and displayed to the user.

4. **Transaction creation and broadcast**: Specify the amount to send, create a transaction, and save it as a PSBT for later signing and broadcasting.

## Usage
### 1. Wallet Initialization
When the Tauri application is launched, the wallet is initialized using a mnemonic stored in environment variables.

### 2. Sync with the Testnet
The wallet syncs with the Testnet in the background. Once the synchronization is complete, a message saying "Connected!" will be displayed.

### 3. Fetch Bitcoin Price
The Bitcoin price is automatically updated every minute and displayed on the wallet interface.

### 4. Create a Transaction
Enter the amount to send and the target rate to create a transaction. The transaction is saved as a PSBT and can be signed and broadcast later.

### 5. Sign and Broadcast a Transaction
The created PSBT is signed and broadcast to the Bitcoin network.

## Roadmap
1. **Add transaction cancellation feature**: Introduce a feature that allows transactions to be canceled after a set time limit, preventing transaction stagnation due to long waiting periods. This will enable more flexible transactions for both buyers and sellers.

2. **Development of Discreet Log Contracts (DLC)**: Develop DLCs utilizing Bitcoin's escrow functionality, allowing goods and services to be provided in advance. This will help resolve dissatisfaction caused by transaction delays and ensure smoother execution of transactions for both buyers and sellers.





# Limit Wallet - ビットコイン指値取引ウォレット

## モチベーション
Limit Walletの主な目的は、ビットコインの流動性を高めることです。ビットコインはデフレ型の資産であり、多くのユーザーがその価値保存手段としてHODL（ホールド）しがちです。このウォレットは、**「時間」**という付加価値を取り入れることで、決済（価値交換）の意欲を促す仕組みを提供しています。

具体的には、買い手は希望するビットコインの目標レートに到達するまでの**「時間」**を「購入」する形となり、ビットコインの価格が希望レートに達した時点で自動的に決済が行われます。これにより、ビットコインの保有者が適切なタイミングで価値を交換しやすくなり、ビットコインの流動性が向上します。

一方で、売り手にとっても、決済が完了するまでにビットコインの価値が上昇する可能性があり、**「時間の喪失」と引き換えに増加した価値を享受する**ことができるというメリットがあります。この仕組みによって、ビットコインをHODLするだけでなく、効率的に決済に使うインセンティブが生まれます。

## 概要
Limit Walletは、Bitcoinのテストネットワークに同期し、ユーザーがビットコインの残高を確認し、トランザクションを作成・署名・ブロードキャストすることができるウォレットです。このウォレットは、Tauri、BDK（Bitcoin Dev Kit）、およびSvelteを使用して構築されています。

## 特徴
- **ビットコインのテストネットワークとの同期**: テストネットに接続してウォレットを同期し、残高の確認ができます。
- **部分署名付きトランザクション（PSBT）の作成・署名**: PSBTの作成、署名、保存、ブロードキャストをサポートします。
- **ビットコインレートの取得**: 1分ごとにAPIからビットコインの現在の価格を取得します。
- **ウォレットの自動同期**: 残高が変動した際には自動的にフロントエンドに通知されます。

## 主要な機能
1. **ウォレットの作成・初期化**: 環境変数からニーモニックを取得し、拡張秘密鍵(XPRV)を生成してウォレットを作成します。

2. **テストネットとの同期**: Electrumサーバーを介してウォレットとテストネットを同期します。同期はバックグラウンドで実行され、同期完了時にはフロントエンドに通知されます。

3. **ビットコインのレート取得**: 1分ごとにビットコインの最新レートをAPIから取得し、ユーザーに表示します。

4. **トランザクションの作成とブロードキャスト**: 送金額を指定してトランザクションを作成し、PSBT形式で部分署名されたトランザクションを保存、またはネットワークにブロードキャストします。

## 使い方
### 1. ウォレットの初期化
Tauriアプリケーションを起動すると、環境変数に保存されたニーモニックをもとにウォレットが初期化されます。

### 2. テストネットとの同期
ウォレットは、バックグラウンドでテストネットに同期します。同期が完了すると、"Connected!"というメッセージが表示されます。

### 3. ビットコインレートの取得
ビットコインのレートは1分ごとに自動更新され、ウォレット画面に表示されます。

### 4. トランザクションの作成
送金額と目標レートを入力してトランザクションを作成します。トランザクションはPSBTとして保存され、後から署名・ブロードキャストが可能です。

### 5. トランザクションの署名・ブロードキャスト
作成されたPSBTは署名され、ビットコインネットワークにブロードキャストされます。

## ロードマップ
1. **取引キャンセル機能の追加**: 制限時間を設けて取引をキャンセルできる機能を導入し、長時間の待機による取引の停滞を防ぎます。これにより、買い手と売り手の双方が市場のタイミングに合わせた柔軟な取引ができるようになります。

2. **Discreet Log Contracts (DLC) の開発**: ビットコインのエスクロー機能を活用したDLCを開発し、商品やサービスの先行提供を可能にします。これにより、取引が遅延した場合でも、売り手と買い手双方の不満を解消し、取引のスムーズな実行を保証します。
