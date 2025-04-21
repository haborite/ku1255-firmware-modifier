# KU-1255 Firmware Modifier

**Lenovo ThinkPad Compact USB キーボード with トラックポイント**（日本語モデル: **0B47208**）のファームウェアをカスタマイズするためのシンプルなGUIツールです。  
任意のキーを別のキーに割り当て直すことができ、たとえば左下の `Fn` キー位置に `Ctrl` キーを割り当てるといったカスタマイズが可能です。

![GUI Overview](https://github.com/haborite/ku1255-firmware-modifier/blob/main/python_ver/img/gui-overview-new.png)

変更内容はキーボードのファームウェアに直接書き込まれるため、**PC側の設定変更は不要**です。接続するすべてのデバイスやOSで同じレイアウトが反映されます。

---

## ✅ 動作環境

本アプリは公式ファームウェアインストーラーを使用するため、本アプリの実行には[こちらのソフトウェア要件](https://support.lenovo.com/jp/ja/solutions/pd026745-thinkpad-compact-usb-keyboard-with-trackpoint-overview-and-service-parts)を満たしている必要があります（Windowsのみ）。

一度ファームウェアを書き込めば、そのキーボード自体は**いずれの主要なOSでも問題なく動作**します。

---

## 🚀 ダウンロードと実行方法

1. [Releasesページ](https://github.com/haborite/ku1255-firmware-modifier/releases)から最新バージョンの `ku1255-firmware-modifier` をダウンロード
2. ダウンロードした `.zip` ファイルを解凍
3. `ku1255-firmware-modifier.exe` を実行

---

## 🖥️ 画面の説明

1. **Keyboard 選択**  
   お使いのキーボードモデルを選択します。日本語版のJIS配列の場合は以下を選択：  
   `0B47208 (89 keys - JIS)`

2. **Language 言語選択**  
   使用する言語を選びます。日本語設定で使う場合は `JP / Japanese` を選択。

3. **Main Layer**  
   通常時のキーマップを設定します。変更したいキーをクリックし、割り当てたいキーをドロップダウンから選択します。

4. **2nd Layer**  
   **Mod** キーと同時押ししたときのキー挙動を定義します。  
   - 初期状態ではModキーがMain Layerに存在しないため、このレイヤーは無効です。  
   - Modキーの位置はMain Layerと2nd Layerで同じである必要があります。

5. **Load config**  
   `.json` 形式の保存済みキーマップを読み込みます。

6. **Save config**  
   現在のキーマップを `.json` ファイルとして保存します。

7. **Install firmware**  
   現在の設定をキーボードに書き込みます。  
   書き込み前にキーボードが接続されていることを確認してください。  
   書き込み後にキーボードを一度USBから外し、再接続することで設定が反映されます。

---

## 🔧 使用例：FnキーとCtrlキーを入れ替える

1. `Load config` をクリックし、次のファイルを開く：  
   `example/Swap-Fn-Ctrl.json`
2. **Main Layer** 上で `Fn` と `Left Ctrl` の位置が入れ替わっていることを確認します。  
   （入れ替わったキーは青くハイライトされます。）
3. `Install firmware` をクリック

   ![Firmware Installer Window](https://github.com/user-attachments/assets/785abfd8-7b13-44aa-b505-b227ed7be4a9)

4. ファームウェアインストーラーが起動したら **Start** をクリック
5. インストールが完了したらインストーラーを閉じます
6. キーボードをUSBから一度取り外し、再接続すると新しい設定が有効になります

---

# 謝辞

本プロジェクトで採用しているファームウェア解析結果は、以下のスレッドでの議論をもとにしています：  
- https://github.com/lentinj/tp-compact-keyboard/issues/32

USB HID Usage IDの対応表はこちらを参考にしています：  
- https://bsakatu.net/doc/usb-hid-to-scancode/

---

このアプリは多言語キーボードへの対応を視野に入れて設計されています。  
他言語版の設定ファイル対応追加を歓迎します。
