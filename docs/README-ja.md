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

- 現時点では、本アプリはMS Windows上でのみ動作します。要望があれば、Linux版およびmacOS版も開発予定です。
- MS Windows上での動作にはMicrosoft Visual C++ 再頒布可能パッケージがインストールされていることが必要です。 
- 一度ファームウェアを書き込めば、そのキーボード自体は**いずれの主要なOSでも問題なく動作**します。
- 公式ファームウェアインストーラーをダウンロードするために、初回起動時にインターネット接続が必要です。

## 🚀 ダウンロードと実行方法

1. [最新バージョン](https://github.com/haborite/ku1255-firmware-modifier/releases/latest)の `ku1255-firmware-modifier.zip` をダウンロード
2. ダウンロードした `.zip` ファイルを解凍
3. `ku1255-firmware-modifier.exe` を実行
(「Windows によって PC が保護されました」「Microsoft Defender SmartScreen は認識されないアプリの起動を停止しました。このアプリを実行すると、PC が危険にさらされる可能性があります」という警告が出る場合は、「詳細情報」をクリック --> 「実行」を選択)

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
   トラックポイント速度を設定します（デフォルト: 1）。トラックポイント速度を調節する際は、まずはLenovo公式ドライバ設定のポインタ速度およびOSのマウス速度設定を優先することをお勧めします。それらで満足な速度が得られない場合に、このファームウェアカスタマイズを検討してください。

8. **Enable middle button click**  
   通常、MS Windows上で公式ドライバを使うと中ボタンクリックが無効化されてスクロール専用アイコンになりますが、
   このチェックボックスをONにすると、中ボタンクリック判定を有効化します（通常のマウスの中ボタンと同じ挙動になる）。

9. **Fn / Media trigger**  
   選択したキーに対して（本来の動作に加えて）Fnキーとしての機能も付与することが出来ます。

10. **Load config**  
   `.json` 形式の保存済みキーマップを読み込みます。

11. **Save config**  
   現在のキーマップを `.json` ファイルとして保存します。


12. **Install firmware**  
   現在の設定をキーボードに書き込みます。  
   書き込み前にキーボードが接続されていることを確認してください。  
   書き込み後にキーボードを一度USBから外し、再接続することで設定が反映されます。

## 🔧 使用例：FnキーとCtrlキーを入れ替える

1. `Load config` をクリックし、次のファイルを開く： `examples/Swap-Fn-Ctrl.json`
2. **Main Layer** 上で `Fn` と `Left Ctrl` の位置が入れ替わっていることを確認します。  
   （入れ替わったキーは青くハイライトされます。）
3. `Install firmware` をクリック
4. ファームウェアインストーラーが起動したら **Start** をクリック
5. インストールが完了したらインストーラーを閉じます
6. キーボードをUSBから一度取り外し、再接続すると新しい設定が有効になります

## ⚠️ 同時押し制限

KU-1255の電気配線上のキーマトリクスは下表の通りです（ANSI論理配列表記で記述しています）.
３つのキーを同時押しした際に、もしそのうち一つのキーの列・行がともに他の２キーと重複する場合, 最後に押したキーは認識されなくなります (ゴーストキーを防ぐため）。

|  | P1.0 | P1.1 | P1.2 | P1.3 | P1.4 | P1.5 | P1.6 | P1.7 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| P0.3 | UP | (none) | (none) | END | FN | PAUSE | LEFT | KP\_MEMSTORE |
| P0.4 | (none) | HOME | (none) | F11 | (none) | (none) | DOWN | (none) |
| P0.5 | F5 | F9 | INTERNATIONAL3 | F10 | BACKSLASH | RETURN | SPACE | BACKSPACE |
| P0.6 | (none) | DELETE | (none) | INSERT | PRINTSCREEN | PAGEUP | PAGEDOWN | (none) |
| P2.0 | F4 | F2 | E | 3 | D | C | (none) | F3 |
| P2.1 | H | 6 | U | 7 | J | M | N | Y |
| P2.2 | ESCAPE | GRAVE | Q | 1 | A | Z | INTERNATIONAL5 | TAB |
| P2.3 | (none) | (none) | (none) | (none) | (none) | RSHIFT | (none) | LSHIFT |
| P4.0 | NONUSBACKSLASH | F1 | W | 2 | S | X | (none) | CAPSLOCK |
| P4.1 | G | 5 | R | 4 | F | V | B | T |
| P4.2 | INTERNATIONAL4 | F8 | O | 9 | L | PERIOD | INTERNATIONAL2 | F7 |
| P4.3 | APOSTROPHE | MINUS | P | 0 | SEMICOLON | NONUSHASH | SLASH | LEFTBRACKET |
| P4.4 | F6 | EQUALS | I | 8 | K | COMMA | INTERNATIONAL1 | RIGHTBRACKET |
| P4.5 | (none) | LCTRL | (none) | (none) | (none) | RCTRL | (none) | (none) |
| P4.6 | LALT | KP\_MEMSUBTRACT | (none) | (none) | (none) | (none) | RALT | (none) |
| P4.7 | (none) | (none) | KP\_MEMCLEAR | F12 | (none) | (none) | RIGHT | LGUI |

---

# 謝辞

本プロジェクトで採用しているファームウェア解析結果は、以下のスレッドでの議論をもとにしています：  
- https://github.com/lentinj/tp-compact-keyboard/issues/32

USB HID Usage IDの対応表はこちらを参考にしています：  
- https://bsakatu.net/doc/usb-hid-to-scancode/

---

このアプリは多言語キーボードへの対応を視野に入れて設計されています。  
他言語版の設定ファイル対応追加を歓迎します。
