# Ruskで記述したデータ構造

`lib/data_structures/` は各データ構造の標準APIと実装を定義します。自動読み込みされるため、`include`は不要です。ファイル名と型名は[shogo-library](https://github.com/shogo314/shogo-library)を参考にしています。

構造体のフィールドは公開されていますが、通常は以下の関数を通して操作します。内部の配列や長さを直接変更するとデータ構造の前提が崩れます。関数への受け渡しは共有なので、更新は呼び出し元の状態にも反映されます。独立した状態が欲しい場合は `let other = value;` を使います。要素のコピーは`__copy__`に従い、手動コピーが共有を残す場合、その共有も保持されます。

## Union-Find

```rusk
let uf := UnionFind(4);
wt uf.unite(0, 1), uf.unite(1, 0); // Yes No
uf.unite(1, 2);
wt uf.same(0, 2), uf.same(0, 3); // Yes No
let independent = uf;
independent.unite(2, 3);
wt uf.same(0, 3), independent.same(0, 3); // No Yes
```

| 呼び出し | 結果・内容 |
|---|---|
| `UnionFind(n: Int) -> UnionFind` | 頂点 `0..n` の集合を作る。`n >= 0` |
| `leader(uf: UnionFind, vertex: Int) -> Int` | 代表頂点を返す。経路圧縮で状態を更新する |
| `unite(uf: UnionFind, u: Int, v: Int) -> Bool` | 異なる集合を併合したときは `true`。既に同じ集合なら `false` |
| `same(uf: UnionFind, u: Int, v: Int) -> Bool` | 同じ集合かを返す。代表探索で状態を更新する |

頂点は `0 <= vertex < n`。構築はO(n)、サイズによる併合と経路圧縮により一連の操作は償却O(α(n))です。コピーはO(n)です。存在しない頂点へのアクセスは配列の範囲検査で失敗します。

## Fenwick Tree

以下の`T`は格納する要素型を表します。呼び出しでは配列・ゼロ元・更新値から推論し、明示する場合は`FenwickTree::<Int>(4, 0)`のように`::<...>`を使います。

```rusk
let fw := FenwickTree([1, 2, 3, 4], 0);
wt fw.prefix_prod(3), fw.prod(1, 4); // 6 9
fw.apply(1, 10);
wt fw.prod(1, 3); // 15
let empty := FenwickTree(0, 0);
wt empty.prod(0, 0); // 0
```

| 呼び出し | 結果・内容 |
|---|---|
| `FenwickTree(n, zero) -> FenwickTree<T>` | `n` 要素をゼロ元で初期化。`n >= 0` |
| `FenwickTree(values, zero) -> FenwickTree<T>` | 初期配列から構築 |
| `fw.apply(position, delta)` | 位置に `delta` を加算 |
| `fw.prefix_prod(end) -> T` | 半開区間 `[0, end)` の集計 |
| `fw.prod(left, right) -> T` | 半開区間 `[left, right)` の集計 |

更新位置は `0 <= position < n`、区間は `0 <= left <= right <= n`。空区間はゼロ元です。不正な位置・区間は `assert` で失敗します。初期配列と内部状態は独立するため、構築後に元の配列を書き換えても木は変わりません。

`T` は可換な加算と減算、コピーを提供し、`zero` はそのゼロ元である必要があります。加減算は引数を変更せず独立した結果を返すように定義します。処理系は可換性などの数学的性質を証明しません。

構築はO(n)、更新・区間集計はO(log n)、コピーはO(n)。要素型の演算・コピーを定数時間とした計算量です。整数の加算・減算には通常どおりオーバーフロー検査が適用されます。

## 区間和セグメント木

```rusk
let seg := SumSegTree([1, 2, 3, 4], 0);
wt seg.prod(1, 4); // 9
seg.set(2, 10);
wt seg.prod(0, 4); // 17
let words := SumSegTree(["ab", "cd", "ef"], "");
wt words.prod(0, 3); // abcdef
words.set(1, "X");
wt words.prod(0, 3); // abXef
```

| 呼び出し | 結果・内容 |
|---|---|
| `SumSegTree(values, zero) -> SumSegTree<T>` | 配列から構築 |
| `seg.set(position, value)` | 要素を `value` のコピーで置換 |
| `seg.prod(left, right) -> T` | 半開区間 `[left, right)` を左から右へ集計 |

更新位置は `0 <= position < n`、区間は `0 <= left <= right <= n`。空配列の構築と空区間の集計も可能です。空区間は単位元を返します。不正な位置・区間は `assert` で失敗します。

`T` は結合的な `+` とコピーを提供し、`zero` はその単位元です。`+` は引数を変更せず独立した結果を返すように定義します。可換である必要はなく、文字列連結のように順序を持つ演算にも使えます。任意の演算関数を引数に取る汎用セグメント木ではないため、最小値などを扱いたい場合は演算を包む型や別の実装を用意します。

構築はO(n)、点更新・区間集計はO(log n)、コピーと保存領域はO(n)。文字列の連結などは要素の長さに応じた演算・コピーのコストが加わります。

## 遅延セグメント木

`LazySegTree` は集計演算に加え、区間へ作用を適用する写像と作用の合成関数を受け取ります。次の例では区間加算と区間和を扱います。

```rusk
fn sum_op(a: Int, b: Int) -> Int { return a + b; }
fn add_mapping(action: Int, value: Int, length: Int) -> Int {
    return value + action * length;
}
fn add_composition(newer: Int, older: Int) -> Int {
    return newer + older;
}
let seg := LazySegTree([1, 2, 3, 4], 0, 0, sum_op, add_mapping, add_composition);
seg.apply_range(1, 4, 5);
wt seg.prod(0, 4); // 25
seg.set(2, 0);
wt seg.get(2), seg.prod(0, 4); // 0 17
```

| 呼び出し | 結果・内容 |
|---|---|
| `LazySegTree(values, identity, identity_action, operation, mapping, composition)` | 遅延セグメント木を構築 |
| `seg.apply_range(left, right, action)` | 半開区間へ作用を適用 |
| `seg.prod(left, right) -> S` | 半開区間の集計 |
| `seg.get(position) -> S` | 指定位置の値のコピー |
| `seg.set(position, value)` | 指定位置の値をコピーで置換 |

`operation(a,b)`は結合的で、`identity`を単位元とします。`mapping(action,value,length)`は長さ`length`の区間値へ作用を適用し、`composition(newer,older)`は古い作用の後に新しい作用を行う合成を返します。`identity_action`は写像を変化させません。これらの代数法則は利用者が満たす必要があります。Ruskは関数の型を検査しますが、法則自体は証明しません。

区間は`0 <= left <= right <= n`、位置は`0 <= position < n`です。構築・点更新・区間作用・区間集計はO(n)、O(log n)、O(log n)、O(log n)。集計値・作用の各演算とコピーを定数時間とした計算量です。内部配列・作用状態は直接変更しないでください。

## 自作の型に演算を与える

通常のRusk関数として演算フックを定義できます。次は整数の重みを包む構造体です。構造体の `copy` は自動生成されます。

```rusk
struct Weight { value: Int; }
fn Weight(value: Int) -> Weight { return Weight { value: value }; }
fn __add__(a: Weight, b: Weight) -> Weight { return Weight(a.value + b.value); }
fn __sub__(a: Weight, b: Weight) -> Weight { return Weight(a.value - b.value); }
let fw := FenwickTree([Weight(2), Weight(5)], Weight(0));
fw.apply(0, Weight(3));
wt fw.prod(0, 2).value; // 10
```

構造体全体の出力は自動生成しないため、例では結果の `.value` を出力します。利用に必要な演算は型を具体化したときに検査します。演算フックの名前は [演算と共有](../reference/index.md) と標準ライブラリの実装も参照してください。

## RollbackUnionFind

`RollbackUnionFind(n)` は `0 <= v < n` の頂点をそれぞれ独立した成分にします。`unite(u, v)` は併合したときだけ `true` を返し、履歴を一つ進めます。`leader`、`same`、`size` は履歴を変更しません。

```rusk
let uf := RollbackUnionFind(4);
uf.unite(0, 1);
let before := uf.snapshot();
uf.unite(1, 2);
wt uf.size(0);       // 3
uf.rollback(before);
wt uf.same(0, 2);    // No
```

`snapshot()` は現在の履歴の長さ、`rollback(checkpoint)` はその長さまでの復元です。チェックポイントは同じインスタンスの現在の履歴の接頭辞を指すものを使います。巻き戻しで破棄した枝のチェックポイントは呼び出し側で捨てます。同じ長さの新しい枝との区別は呼び出し側が管理します。負の頂点数、不正な頂点、現在の履歴の長さを越えるか負のチェックポイントは実行時エラーです。頂点数0では履歴位置0の取得・復元ができます。

構築は `O(n)`、サイズによる併合を使うため頂点の検索と併合は `O(log n)`、スナップショットは `O(1)`、`k` 件の併合の巻き戻しは `O(k)` です。コピーすると親配列と履歴が独立します。

## 永続セグメント木

`PersistentSegTree(values, identity, operation)` は値をコピーして初版を作ります。`operation` は結合的で、`identity` を単位元とします。版をまたいで結果が一貫するよう、演算は引数の値だけで結果を決める関数を指定します。

```rusk
let first := PersistentSegTree(["a", "b", "c"], "", (a: Str, b: Str) => a + b);
let second := first.updated(1, "X");
wt first.prod(0, 3), second.prod(0, 3);  // abc aXc
wt second.get(1);                        // X
```

`updated(index, value)` は指定位置を置き換えた新しい版を返します。`prod(l, r)` は半開区間を左から右へ集計し、空区間では単位元のコピーを返します。`get(index)` も値のコピーを返します。不正な添字や区間は実行時エラーです。空の入力からも構築でき、`prod(0, 0)` が使えます。

構築は `O(n)`、一点更新と区間集計は `O(log n)` です。演算と値のコピーにかかる時間は別に数えます。一点更新では根から葉までの `O(log n)` 個のノードを作ります。変更されない不変の部分木は版の間で共有します。ハンドルをコピーするとこの共有も維持されます。
