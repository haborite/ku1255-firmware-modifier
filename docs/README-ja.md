# KU-1255 Firmware Modifier

**[Lenovo ThinkPad Compact USB キーボード with トラックポイント](https://support.lenovo.com/jp/ja/solutions/pd026745-thinkpad-compact-usb-keyboard-with-trackpoint-overview-and-service-parts)**（日本語モデル: **0B47208**）のファームウェアをカスタマイズするためのシンプルなGUIツールです。  

任意のキーを別のキーに割り当て直すことができます。たとえば左下の `Fn` キー位置に `Ctrl` キーを割り当てるといったカスタマイズが可能です。
さらに、以下のより高度な機能を利用することが出来ます。
- レイヤー機能: Modキーとの同時押しによりキーの種類を変更
- キーマクロ機能: Ctrl, Shift, Alt, Winキーとの同時押しを単一キーで代替
- メディアキー: 音量調節や動画再生停止等の特殊なキーの割り当て
- トラックポイント速度変更: トラックポイントの加速度をLenovo公式ドライバの限界よりも大きな値に設定可能

変更内容はキーボードのファームウェアに直接書き込まれるため、**PC側の設定変更は不要**です。接続するすべてのデバイスやOSで同じレイアウトが反映されます。

![GUI Overview](https://github.com/haborite/ku1255-firmware-modifier/blob/main/docs/gui-overview.png)

---

## 📜 対応機種
**[Lenovo ThinkPad Compact USB キーボード with トラックポイント (KU-1255)](https://support.lenovo.com/jp/ja/solutions/pd026745-thinkpad-compact-usb-keyboard-with-trackpoint-overview-and-service-parts)**

**販売部品番号** : 0B47190, 0B47191, 0B47192, 0B47194, 0B47195, 0B47197, 0B47198, 0B47200, 0B47201, 0B47202, 0B47204, 0B47205, 0B47206, 0B47207, 0B47208, 0B47209, 0B47210, 0B47211, 0B47212, 0B47213, 0B47215, 0B47216, 0B47217, 0B47218, 0B47219, 0B47220, 0B47221, 0B47222, 0B47223, 0B47224, 0B47225

## ✅ 動作環境

- **Windows** 10 / 11 (x64)
- **Linux** (x64): Debian / Ubuntu 系は `.deb` パッケージ、その他のディストリビューションは AppImage
- **macOS**: 未対応（Apple Silicon 向けのビルドはありますが、書き込みの動作確認が済んでいません）
- 一度ファームウェアを書き込めば、そのキーボード自体は**いずれの主要なOSでも問題なく動作**します。
- 公式ファームウェアインストーラーをダウンロードするために、初回起動時にインターネット接続が必要です。

## 🚀 ダウンロードと実行方法

[最新リリース](https://github.com/haborite/ku1255-firmware-modifier/releases/latest)からお使いのOS用のファイルをダウンロードします。

**Windows**
1. インストーラー（`*-setup.exe`）をダウンロードして実行
    - 「Windows によって PC が保護されました」（Microsoft Defender SmartScreen）という警告が出る場合は、「詳細情報」をクリック --> 「実行」を選択
    - WebView2 ランタイムがない場合（Windows 10/11 ではまれ）は、インストーラーが自動でダウンロードします
2. スタートメニューから **Ku1255FirmwareModifier** を起動

**Linux（Debian / Ubuntu 系）**
1. `.deb` ファイルをダウンロードしてインストール
    ```
    sudo apt install ./ku1255-firmware-modifier_*_amd64.deb
    ```
    root 権限なしでキーボードに書き込むための udev ルールも一緒にインストールされます。
2. キーボードを一度取り外して再接続し、アプリケーションメニューから起動

**Linux（その他のディストリビューション）**
1. `.AppImage` ファイルをダウンロードし、実行権限を付与： `chmod +x ku1255-firmware-modifier*.AppImage`
2. root 権限なしで書き込むために [udev ルール](../packaging/linux/70-ku1255.rules) をインストール
    ```
    sudo cp 70-ku1255.rules /etc/udev/rules.d/
    sudo udevadm control --reload && sudo udevadm trigger
    ```
3. AppImage を実行

**ファイルの保存場所**  
ダウンロードした公式インストーラー、設定ファイルのサンプル、ファームウェアの中間ファイルは次の場所に保存されます。
- Windows: `%LOCALAPPDATA%\ku1255-firmware-modifier`
- Linux: `~/.local/share/ku1255-firmware-modifier`

## 🖥️ 画面の説明

![Interface Overview](https://github.com/haborite/ku1255-firmware-modifier/blob/main/docs/interface-overview.png)

1. **Keyboard 選択**  
   お使いのキーボードモデルを選択します。日本語版のJIS配列の場合は以下を選択： `0B47208 (89 keys - JIS)`

2. **Language 言語選択**  
   使用する言語を選びます。日本語設定で使う場合は `JP / Japanese` を選択。

3. **Main Layer**  
   通常時のキーマップを設定します。変更したいキーをクリックし、割り当てたいキーをドロップダウンから選択します。

4. **2nd Layer**  
   **Mod** キーと同時押ししたときのキー挙動を定義します。  
   - 初期状態ではModキーがMain Layerに存在しないため、このレイヤーは無効です。  
   - Modキーの位置はMain Layerと2nd Layerで同じである必要があります。

5. **Macro keys**
   任意のキーを、Ctrl, Shift, Alt, Winキーの組み合わせたキーマクロを作成することが出来ます（最大24種類）。
   設定したキーマクロ（Macro01 - 24）をMainまたは2nd Layerにマッピング出来ます。

6. **Media keys**  
   音量やディスプレイの明るさ調整等の機能を担うメディアキーを設定することが出来ます（最大11種類）。
   設定したメディアキー（Media01 - 11）をMainまたは2nd Layerにマッピング出来ます。

7. **TrackPoint Speed**  
   トラックポイント速度を設定します。トラックポイントの傾きに対するカーソル速度を個別に設定できます。簡単なプリセット設定も利用できます。
   Speed 1からSpeed 9は、ドライバ設定画面上の9段階の速度設定に対応しています。公式ドライバがインストールされていない環境（macOSなど）ではSpeed 5が使われます。
   ![Trackpoint Settings](https://github.com/haborite/ku1255-firmware-modifier/blob/main/docs/trackpoint-overview.png)

8. **Others**  
   - **Fn / Media trigger**: 選択したキーに対して（本来の動作に加えて）Fnキーとしての機能も付与することが出来ます。
   - Force to disable Thinkpad preffered scrolling

9. **Load config**  
   `.json` 形式の保存済みキーマップを読み込みます。

10. **Save config**  
   現在のキーマップを `.json` ファイルとして保存します。


11. **Install firmware**  
   現在の設定をキーボードに書き込みます。  
   書き込み前にキーボードが接続されていることを確認してください。  
   書き込み後にキーボードを一度USBから外し、再接続することで設定が反映されます。

## 🔧 使用例：FnキーとCtrlキーを入れ替える

1. `Load config` をクリックし、次のファイルを開きます： `examples/Swap-Fn-Ctrl.json`
2. 左上の**Keyboard selection**でキーボードの種類を選びます（JIS or ANSI など）。
3. **Main Layer** 上で `Fn` と `Left Ctrl` の位置が入れ替わっていることを確認します。  
   （入れ替わったキーは青くハイライトされます。）
4. `Install firmware` をクリック
5. インストール画面で **Start** をクリック
6. 「Flashing finished」と表示されたら **Finish** をクリック
7. キーボードをUSBから一度取り外し、再接続すると新しい設定が有効になります

## ♻️ ファームウェアのリセット
- ファームウェアは繰り返し書き込み可能です
- 書き込みが途中で失敗した場合、キーボードはファームウェア更新モードで起動します。もう一度インストールを実行してください（**Retry**）。
- [Lenovo公式ファームウェアインストーラー](https://support.lenovo.com/jp/ja/solutions/pd026745-thinkpad-compact-usb-keyboard-with-trackpoint-overview-and-service-parts)を実行すると、ファームウェアが出荷時の状態に戻ります。


## 開発
devlopment guideをご覧ください

https://github.com/haborite/ku1255-firmware-modifier/tree/main/dev#readme


---

# 謝辞

本プロジェクトで採用しているファームウェア解析結果は、以下のスレッドでの議論をもとにしています：  
- https://github.com/lentinj/tp-compact-keyboard/issues/32
- https://hohlerde.org/rauch/en/elektronik/projekte/tpkbd-fix

USB HID Usage IDの対応表はこちらを参考にしています：  
- https://bsakatu.net/doc/usb-hid-to-scancode/

---

このアプリは多言語キーボードへの対応を視野に入れて設計されています。  
他言語版の設定ファイル対応追加を歓迎します。
