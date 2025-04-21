# 概要
Lenovo製ThinkPadトラックポイントキーボード KU-1255 (日本語版の型番: 0B47208)のファームウェアを簡単に編集するためのGUIアプリです。
キーボード上の任意のキー配置を変更することが出来ます。たとえば、左下のFnキーの位置にCtrlキーを割り当てたい、といった需要に応えることができます。
ファームウェアはキーボード本体にインストールされるため、PC側の設定は一切必要ありません。

<img width="960" alt="gui-overview" src="https://github.com/haborite/ku1255-firmware-modifier/blob/main/python_ver/img/gui-overview-new.png">

# 実行可能環境
公式ファームウェアを利用するため、[公式ファームウェアダウンロードページ](https://support.lenovo.com/jp/ja/solutions/pd026745-thinkpad-compact-usb-keyboard-with-trackpoint-overview-and-service-parts)における「ソフトウェア要件」に準拠します。
ファームウェアを書き込み後のキーボード自体はほとんどのOSで動作します。

# ダウンロード・実行方法
1. Relaseから最新版のku1255-firmware-modifierをダウンロード
2. ダウンロードしたzipファイルを解凍
3. ku1255-firmware-modifier.exeを実行

# 画面説明
1. Keyboard 選択欄: お使いのキーボードを選びます。日本語版の場合は「0B47208 (89 keys - JIS)」。
2. Language 選択欄: 設定言語を選びます。日本語キーボードとして使う場合は「JP / Japanese」。
3. Main Layer: 通常時におけるキー配置を設定します。設定したいキーを押して現れる選択欄から、変更後のキーを選択します。
4. 2nd Layer : 「Mod」キーと同時押しした際のキー配置を設定します。デフォルトの状態では「Mod」キーMain Layerに存在しないため、機能がオフになっています。「Mod」キーの位置はMain Layerと2nd Layerで同一である必要があります。
5. Load config: 保存済みのキー配置設定を読み込みます。
6. Save config: 現在のキー配置設定を保存します。
7. Install firmware: 現在の設定をキーボードに書き込みます。キーボードが接続された状態で実行してください。インストールが完了したのちに、USBを一度抜き、再度差し込むことで設定が反映されます。

# 使用例

## 1. FnキーとCtrlキーを入れ替えたい
1. "Load config"から"example\Swap-Fn-Ctrl.json"をひらく
2. "Main Layer"でFnとLCtrlの位置が入れ替わっている（キーが青色にマークされる）ことを確認
3. "Install firmware"を押す

<img width="360" alt="firmware installer window" src="https://github.com/user-attachments/assets/785abfd8-7b13-44aa-b505-b227ed7be4a9">

4. ファームウェアインストーラーが起動するので、「Start」を選択
5. インストールが完了したら、ファームウェアインストーラーウィンドウを閉じる
6. キーボードをUSBコネクタから一度取り外し、再度接続する。
