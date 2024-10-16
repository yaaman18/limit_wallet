<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/tauri";
  import { listen } from '@tauri-apps/api/event';

  let bitcoinRate = 0;
  let amountToSend = "";
  let targetRate = "";
  let walletInfo: {
    network: string;
    descriptor: string;
    balance: number;
    address: string;
  } | null = null;
  let errorMessage = "";
  let mnemonicStr = "";
  let walletMessage = "";
  let syncMessage = "";
  let balance = 0;
  let receiveAddress: string = "";

  // テストネット接続状態を管理する変数
  let isConnected = false;

  /**
   * 環境変数からニーモニックを取得する関数
   */
  async function loadMnemonicFromEnv() {
    try {
      mnemonicStr = await invoke('get_mnemonic_from_env');
    } catch (error) {
      console.error("ニーモニックの取得に失敗しました:", error);
      errorMessage = `ニーモニックの取得に失敗しました: ${error}`;
    }
  }

  /**
   * ウォレットを環境変数のニーモニックから初期化する関数
   */
  async function createWalletFromEnv() {
    try {
      walletMessage = await invoke('create_wallet_from_env');
      await printWalletInfo();
    } catch (error) {
      console.error("ウォレットの初期化に失敗しました:", error);
      errorMessage = `ウォレットの初期化に失敗しました: ${error}`;
    }
  }

  /**
   * テストネットと同期する関数
   */
  async function connectToTestnet() {
    try {
      syncMessage = await invoke('connect_to_testnet');
      console.log("テストネットへの同期に成功しました:", syncMessage);

      // 接続が成功した場合に isConnected を true に設定
      isConnected = true;

      await printWalletInfo();
    } catch (error) {
      console.error("テストネットへの接続に失敗しました:", error);
      errorMessage = `テストネットへの接続に失敗しました: ${error}`;
    }
  }

  /**
   * ウォレット情報を取得して表示する関数
   */
  async function printWalletInfo() {
    try {
      walletInfo = await invoke('print_wallet_info');
      balance = walletInfo?.balance ?? 0; // walletInfo が null でない場合に balance を設定
    } catch (error) {
      console.error("ウォレット情報の取得に失敗しました:", error);
      errorMessage = `ウォレット情報の取得に失敗しました: ${error}`;
    }
  }

  /**
   * ビットコインレートを取得する関数
   */
  async function fetchBitcoinRate() {
    try {
      bitcoinRate = await invoke('fetch_bitcoin_price');
    } catch (error) {
      console.error("ビットコインレートの取得に失敗しました:", error);
      errorMessage = `ビットコインレートの取得に失敗しました: ${error}`;
    }
  }

  /**
   * 新しい受信アドレスを生成する関数
   */
  async function generateNewAddress() {
    try {
      receiveAddress = await invoke('generate_receive_address');
    } catch (error) {
      console.error("新しいアドレスの生成に失敗しました:", error);
      errorMessage = `新しいアドレスの生成に失敗しました: ${error}`;
    }
  }

  /**
   * フォームを送信してトランザクションを作成する関数
   */
  async function submitForm() {
    try {
      if (!amountToSend || !targetRate) {
        alert("送金額と目標レートの両方を入力してください。");
        return;
      }

      // 目標レートを環境変数に保存
      await invoke('save_rate_to_env', { rate: parseFloat(targetRate) });

      console.log(`トランザクション: 送金額: ${amountToSend} satoshis, 目標レート: ¥${targetRate}`);

      // トランザクションを作成
      const psbt = await invoke('create_transaction', {
        to_address: "tb1qlj64u6fqutr0xue85kl55fx0gt4m4urun25p7q", // テスト用アドレス
        amount: parseInt(amountToSend)
      });

      // 作成されたPSBTを環境変数に保存
      await invoke('save_psbt_to_env', { psbt: psbt });

      alert("トランザクションが作成され、PSBTが保存されました！");

    } catch (error) {
      console.error("トランザクションの作成またはPSBTの保存に失敗しました:", error);
      errorMessage = `トランザクションの作成またはPSBTの保存に失敗しました: ${error}`;
    }
  }


  /**
   * アプリケーションのマウント時にニーモニックを環境変数からロード
   */
  onMount(() => {
    (async () => {
      try {
        // ニーモニックのロード
        await loadMnemonicFromEnv();

        // ビットコインレートの取得とウォレットの初期化を同時に実行
        await Promise.all([fetchBitcoinRate(), createWalletFromEnv()]);

        // 残高監視を開始
        await invoke('start_balance_monitor');

        // balance_changed イベントをリッスン
        const unlisten = await listen('balance_changed', (event) => {
          balance = event.payload as number;
        });

        // 1分ごとにビットコインレートを更新するタイマーを設定
        const rateInterval = setInterval(async () => {
          await fetchBitcoinRate();
        }, 60000); // 1分ごとに実行

        // クリーンアップ時にリスナーとタイマーを解除
        return () => {
          unlisten();
          clearInterval(rateInterval); // タイマーの解除
        };
      } catch (error) {
        console.error("onMount 中にエラーが発生しました:", error);
        errorMessage = `onMount 中にエラーが発生しました: ${error}`;
      }
    })();
});

</script>

<main>
  <h1>ビットコインウォレット</h1>

  <p>現在のビットコインレート: ¥{bitcoinRate}</p>
  <p>現在の残高: {balance} satoshis</p>


  <!-- テストネットと同期ボタン -->
  <button on:click={connectToTestnet}>テストネットと同期</button>
  {#if isConnected}
    <p style="color: green;">Connected!</p>
  {/if}

  {#if walletInfo}
    <div>
      <h2>ウォレット情報</h2>
      <p>ネットワーク: {walletInfo.network}</p>
      <p>ディスクリプタ: {walletInfo.descriptor}</p>
      <p>残高: {walletInfo.balance} satoshis</p>
      <p>アドレス: {walletInfo.address}</p>
    </div>
  {/if}

  <!-- 新しい受信アドレス生成ボタン -->
  <button on:click={generateNewAddress}>新しい受信アドレスを生成</button>
  {#if receiveAddress}
    <p>新しい受信アドレス: {receiveAddress}</p>
  {/if}

  <!-- トランザクション作成フォーム -->
  <form on:submit|preventDefault={submitForm}>
    <label for="amount">送金額 (satoshi単位):</label>
    <input
      type="number"
      id="amount"
      bind:value={amountToSend}
      placeholder="送金額（satoshi）"
      required
    />

    <label for="rate">目標レート (円):</label>
    <input
      type="number"
      id="rate"
      bind:value={targetRate}
      placeholder="目標レート（円）"
      required
    />

    <button type="submit">送信</button>
  </form>

  {#if errorMessage}
    <p class="error">{errorMessage}</p>
  {/if}
</main>

<style>
  main {
    font-family: Arial, sans-serif;
    margin: 0 auto;
    padding: 2rem;
    max-width: 600px;
  }

  h1, h2 {
    text-align: center;
  }

  form {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  label {
    font-weight: bold;
  }

  input {
    padding: 0.5rem;
    font-size: 1rem;
  }

  button {
    padding: 0.75rem;
    background-color: #007BFF;
    color: white;
    font-size: 1rem;
    border: none;
    border-radius: 5px;
    cursor: pointer;
  }

  button:hover {
    background-color: #0056b3;
  }

  .error {
    color: red;
    font-weight: bold;
  }
</style>
