# これは？

todo を管理するための TUI アプリケーションです．Rust の学習のために個人的に制作しております．

# 動作環境
- Windows11 Pro (24H2)
- rustc 1.86.0

# できること
|Command|Description|
|:---:|:---|
|`list`|"todo.txt" に書かれた todo の一覧表示|
|`add <todo.txtのフォーマット>`|新しい todo の追加．作成日は自動挿入|
|`done <todo のインデックス>`|todo に完了マークと完了日を挿入|
|`rm <todo のインデックス>`|todo を "tood.txt" から削除|
|`sd`|dueタグを含み，かつまだ完了していない todo を**期日が近い**順にソート|
|`sp`|優先度を含み，かつまだ完了していない todo を**優先度が高い**順にソート|

$\textreferencemark$ rm : remove

$\textreferencemark$ sd : sort deadline

$\textreferencemark$ sp : sort priority

# todo.txt のフォーマット

todo.txt のフォーマットは[こちら](https://github.com/todotxt/todo.txt)を参照してください．

# 今後実装したいこと
- "todo.txt" のインポート・エクスポート
- `help` コマンドの実装
- 各種設定の実装
