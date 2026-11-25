mod support;
use support::Case;

#[test]
fn lazy_segment_tree_composes_range_actions_and_preserves_order() {
    let source = r#"
fn add(a:Int,b:Int)->Int{return a+b;}
fn add_to_sum(action:Int,value:Int,length:Int)->Int{return value+action*length;}
fn compose_add(newer:Int,older:Int)->Int{return newer+older;}
let seg:=LazySegTree([1,2,3,4,5],0,0,add,add_to_sum,compose_add);
seg.apply_range(0,3,2);seg.apply_range(1,5,4);
wt seg.prod(0,2),seg.prod(2,5),seg.prod(0,5),seg.get(1);
seg.set(2,10);wt seg.prod(0,5),seg.get(2);
let detached=seg;detached.apply_range(0,5,1);wt seg.prod(0,5),detached.prod(0,5);
let no_values:Vec<Int>:=[];let empty:=LazySegTree(no_values,0,0,add,add_to_sum,compose_add);
empty.apply_range(0,0,8);wt empty.prod(0,0);
struct Affine { a:Int; b:Int; }
fn Affine(a:Int,b:Int)->Affine{return Affine {a:a,b:b};}
fn affine_map(action:Affine,value:Int,length:Int)->Int{return action.a*value+action.b*length;}
fn affine_compose(newer:Affine,older:Affine)->Affine{return Affine(newer.a*older.a,newer.a*older.b+newer.b);}
let affine:=LazySegTree([1,2,3],0,Affine(1,0),add,affine_map,affine_compose);
affine.apply_range(0,3,Affine(2,1));affine.apply_range(1,3,Affine(3,4));
wt affine.prod(0,3),affine.prod(1,3);
"#;
    for options in [vec![], vec!["--no-opt"]] {
        let out = Case::new(source).run_with_options("", &options);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(out.stdout, b"11 26 37 8\n38 10\n38 43\n0\n47 44\n");
    }
}

#[test]
fn lazy_segment_tree_rejects_invalid_ranges() {
    let source = r#"
fn add(a:Int,b:Int)->Int{return a+b;}
fn mapping(action:Int,value:Int,length:Int)->Int{return value+action*length;}
fn compose(newer:Int,older:Int)->Int{return newer+older;}
"#;
    for statement in [
        "let seg:=LazySegTree([1,2],0,0,add,mapping,compose);seg.apply_range(-1,1,2);",
        "let seg:=LazySegTree([1,2],0,0,add,mapping,compose);seg.apply_range(0,3,2);",
        "let seg:=LazySegTree([1,2],0,0,add,mapping,compose);seg.prod(2,1);",
        "let seg:=LazySegTree([1,2],0,0,add,mapping,compose);seg.set(2,3);",
    ] {
        let out = Case::new(&format!("{source}{statement}")).run("");
        assert_eq!(out.status.code(), Some(1), "{statement}");
        assert!(String::from_utf8_lossy(&out.stderr).contains("assertion failed"));
    }
}

#[test]
fn permutation_enumeration_is_lazy_independent_and_unique() {
    let source = r#"
let input:=[2,1,1];let it:=permutations(input);let saved:Vec<Vec<Int>>:=[];
for p in it {saved.push_back(p);wt p;}
wt input,saved.len();saved[0][0]=9;wt saved[1];
let empty:Vec<Int>:=[];wt permutations(empty).to_vec().len();
let a:=[1,2,3];wt next_permutation(a),a;wt prev_permutation(a),a;wt prev_permutation(a),a;
let original:=permutations([1,2,3]);__iter_next__(original);let detached=original;
wt __iter_next__(original),__iter_next__(detached);
"#;
    for options in [vec![], vec!["--no-opt"]] {
        let out = Case::new(source).run_with_options("", &options);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(out.stdout,b"1 1 2\n1 2 1\n2 1 1\n2 1 1 3\n1 2 1\n1\nYes 1 3 2\nYes 1 2 3\nNo 3 2 1\n1 3 2 1 3 2\n");
    }
}

#[test]
fn big_integers_use_floor_arithmetic_and_independent_copies() {
    let out=Case::new(r#"let a:=BigInt("123456789012345678901234567890");let b=a;a+=BigInt(1);wt a,b;wt BigInt(-7)/BigInt(3),BigInt(-7)%BigInt(3),BigInt(7)/BigInt(-3),BigInt(7)%BigInt(-3);wt BigInt(2)**100;wt BigInt(-3)>>1,BigInt(-3)&BigInt(6),~BigInt(0);wt Int(BigInt("-9223372036854775808"));"#).run("");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(out.stdout,b"123456789012345678901234567891 123456789012345678901234567890\n-3 2 -3 -2\n1267650600228229401496703205376\n-2 4 -1\n-9223372036854775808\n");
}

#[test]
fn packed_arrays_and_bits_have_value_reads_and_independent_copies() {
    let source = r#"
let a:=IntArray([3,1,2]);let held:=a.get(0);a[0]:=4;a.push_back(9);let b=a;a.sort();wt a,b,held,a.sum();
let xs:=a.map((x:Int)=>x*2).to_vec();wt xs;
let bits:=BitVec("10101");bits[1]:=true;wt bits,bits.count(),bits<<2,~bits;
let fixed_bits:=Bits::<5>("00101");wt fixed_bits,fixed_bits>>1,fixed_bits.len(),fixed_bits.to_vec();
"#;
    let out = Case::new(source).run("");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        out.stdout,
        b"1 2 4 9 4 1 2 9 3 16\n2 4 8 18\n10111 4 11100 01000\n00101 00010 5 Yes No Yes No No\n"
    );
}

#[test]
fn deque_and_fixed_array_preserve_shared_elements_and_copy_state() {
    let out=Case::new(r#"
let q:=Deque([1,2]);let value:=3;q.push_front(value);value=4;q.push_back(5);wt q.to_vec(),q.pop_front(),q.pop_back();
let copied=q;q[0]=7;wt q.to_vec(),copied.to_vec();
let a:=Array::<Int,3>([1,2,3]);let held:=a[0];a[0]=9;wt held,a.to_vec();
let b=a;a[1]:=8;wt a.to_vec(),b.to_vec();
let filled:=Array::<Int,2>(4);filled[0]=5;wt filled.to_vec();
fn unit(){}let units:=Deque::<Unit>();units.push_back(unit());units.pop_front();wt units.len();
"#).run("");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        out.stdout,
        b"4 1 2 5 4 5\n7 2 1 2\n9 9 2 3\n9 8 3 9 2 3\n5 4\n0\n"
    );
}

#[test]
fn ordered_sets_multisets_and_maps_iterate_by_key() {
    let out=Case::new(r#"
let s:=OrderedSet::<Int>();wt s.insert(3),s.insert(1),s.insert(3);wt s.to_vec(),s.lower_bound(2)??9,s.upper_bound(3)??9;
let m:=MultiSet::<Int>();m.insert(2);m.insert(1);m.insert(2);wt m.to_vec(),m.count(2),m.erase_one(2),m.to_vec();
let d:=OrderedDict::<Int,Str>();d.set(3,"c");d.set(1,"a");let held:=d.get(1)??"?";d[1]="A";wt d.keys(),d.values(),held;
let c=d;c[1]="X";wt d.values(),c.values();
"#).run("");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        out.stdout,
        b"Yes Yes No\n1 3 3 9\n1 2 2 2 Yes 1 2\n1 3 A c A\nA c X c\n"
    );
}

#[test]
fn container_and_union_find_api_agree_with_their_public_contracts() {
    let program = Case::new(
        r#"
readonly let empty_vec:Vec<Int> := [];
readonly let full_vec := [1];
readonly let empty_str := "";
readonly let full_str := "x";
wt is_empty(empty_vec), full_vec.is_empty(), empty_str.is_empty(), is_empty(full_str);

let multiset:=MultiSet::<Int>();
multiset.insert(2);multiset.insert(2);multiset.insert(2);multiset.insert(1);
wt multiset.erase_one(2),multiset.count(2),multiset.erase_all(2),multiset.erase_all(2),multiset.erase_one(2),multiset.count(1);
let set:=OrderedSet::<Int>();set.insert(2);wt set.erase(2);

fn unite_discarding_result(uf:UnionFind,u:Int,v:Int)->Unit {uf.unite(u,v);}
let uf:=UnionFind(4);
wt uf.unite(0,1),uf.unite(1,0),uf.unite(2,2);
unite_discarding_result(uf,1,2);
wt uf.same(0,2),uf.same(0,3);
"#,
    );
    let removed = Case::new("let m:=MultiSet::<Int>();m.erase(1);");
    for options in [&[][..], &["--no-opt"][..]] {
        let out = program.run_with_options("", options);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(
            out.stdout,
            b"Yes No Yes No\nYes 2 2 0 No 1\nYes\nYes No No\nYes No\n"
        );
        let out = removed.compile_with_options(options);
        assert!(!out.status.success());
        assert!(String::from_utf8_lossy(&out.stderr).contains("no matching function 'erase'"));
    }
}

#[test]
fn selection_sorted_sets_and_numeric_algorithms_match_expected_results() {
    let out=Case::new(r#"
let a:=[5,1,4,2,3];nth_element(a,2);wt a[2];partial_sort(a,3);wt a[0],a[1],a[2];
let b:=[1,1,2,2,3];wt unique(b),b;rotate(b,1);wt b;
let c:=[1,2,3,4];wt partition(c,(n:Int)=>n%2==0),c;
wt merge_sorted([1,3],[2,3]),set_union([1,1,3],[1,2]),set_intersection([1,1,3],[1,2]),set_difference([1,1,3],[1,2]);
wt partial_sum([1,2,3]),adjacent_difference([1,3,6]),inner_product([1,2],[3,4],5);
wt gcd(-9223372036854775808,2),lcm(-6,8),gcd(BigInt("12345678901234567890"),BigInt(30));
"#).run("");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        out.stdout,
        b"3\n1 2 3\n3 1 2 3\n2 3 1\n2 2 4 1 3\n1 2 3 3 1 1 2 3 1 1 3\n1 3 6 1 2 3 16\n2 24 30\n"
    );
}

#[test]
fn slices_share_updates_copy_the_owner_and_keep_iterator_positions_independent() {
    let out=Case::new(r#"
let source:=[1,2,3,4];let view:=slice(source,1,3);let readonly_view:=readonly_slice(view);
source[1]=8;wt view.to_vec(),readonly_view.copy_to_vec();view[1]:=9;wt source;
let detached=view;source[1]=7;wt detached.to_vec(),view.to_vec();
let it:=__iter__(view);__iter_next__(it);let independent=it;wt __iter_next__(it),__iter_next__(independent);
let packed:=IntArray([3,4,5]);let part:=slice(packed,1,3);part[0]:=6;wt packed,part.to_vec();
let q:=Deque([1,2,3]);let qview:=slice(q,1,3);qview[0]=7;wt q.to_vec();
let a:=Array::<Int,2>([1,2]);wt slice(a).to_vec();
"#).run("");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        out.stdout,
        b"8 3 8 3\n1 8 9 4\n8 9 7 9\n9 9\n3 6 5 6 5\n1 7 3\n1 2\n"
    );
}

#[test]
fn bigint_arithmetic_matches_python_integer_fixtures() {
    // Expected values were computed with Python integers, independently of the
    // word representation and division algorithm in the runtime.
    let cases: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("fixtures/bigint-arithmetic.json")).unwrap();
    let mut input = format!("{}\n", cases.len());
    let mut expected = String::new();
    for case in cases {
        input.push_str(case["input"].as_str().unwrap());
        expected.push_str(case["output"].as_str().unwrap());
    }
    let source = r#"in count:Int;for i in 0..count{in left,right:Str;in shift:Int;let a:=BigInt(left);let b:=BigInt(right);wt a+b,a-b,a*b,a/b,a%b,a&b,a|b,a^b,~a,a<<shift,a>>shift,Str(a),a==b,a<b;}"#;
    for options in [vec![], vec!["--no-opt"]] {
        let out = Case::new(source).run_with_options(&input, &options);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(out.stdout, expected.as_bytes());
    }
}

#[test]
fn scalar_conversions_hashes_and_static_lengths_are_consistent() {
    let out=Case::new(r#"
let limit:=Int128("-170141183460469231731687303715884105728");wt Int128(BigInt(limit)),gcd(limit,Int128(2));
let dict:=Dict::<BigInt,Int>();dict.set(BigInt("00012"),7);wt dict.get(BigInt(12))??0,!BigInt(0),Bool(BigInt(-1));
let a:=Bits::<0>(false);wt a.len(),a.count(),a.all(),a.to_vec().len();
let empty:Vec<Int>:=[];let fixed_array:=Array::<Int,0>(empty);wt fixed_array.len();
"#).run("");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        out.stdout,
        b"-170141183460469231731687303715884105728 2\n7 Yes Yes\n0 0 Yes 0\n0\n"
    );
}

#[test]
fn invalid_indices_lengths_and_arithmetic_fail_before_native_undefined_behavior() {
    for (source, message) in [
        ("Int(BigInt(2)**100);", "integer conversion out of range"),
        ("Int128(BigInt(2)**127);", "integer conversion out of range"),
        ("BigInt(1)/BigInt(0);", "division by zero"),
        ("BigInt(1)%BigInt(0);", "division by zero"),
        ("BigInt(\"1x\");", "invalid BigInt input"),
        ("BigInt(2)**-1;", "negative integer exponent"),
        ("let a:=IntArray([1]);a[1];", "array index out of range"),
        ("IntArray(-1,0);", "negative array length"),
        ("Bits::<3>(\"11\");", "bit length mismatch"),
        ("Bits::<-1>(false);", "negative bit length"),
        ("BitVec(\"12\");", "invalid bit string"),
        ("BitVec(\"1\")&BitVec(\"11\");", "bit length mismatch"),
        ("Array::<Int,2>([1]);", "array length mismatch"),
        ("Array::<Int,-1>(0);", "negative array length"),
        ("let q:=Deque::<Int>();q.pop_front();", "empty deque"),
        (
            "let values:=[1];let s:=slice(values);values.push_back(2);s.len();",
            "assertion failed",
        ),
        (
            "let a:=[1];nth_element(a,2);",
            "selection index out of range",
        ),
        ("lcm(9223372036854775807,2);", "integer overflow"),
        (
            "let it:=permutations([1]);__iter_next__(it);__iter_next__(it);",
            "assertion failed",
        ),
    ] {
        let out = Case::new(source).run("");
        assert_eq!(
            out.status.code(),
            Some(1),
            "{source}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(
            String::from_utf8_lossy(&out.stderr).contains(message),
            "{source}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

#[test]
fn readonly_and_value_reads_cannot_silently_mutate_copies() {
    for source in [
        "let a:=IntArray([1]);a[0]=2;",
        "let bits:=BitVec(\"1\");bits[0]=false;",
        "readonly let a:=IntArray([1]);a.set(0,2);",
        "readonly let n:=BigInt(3);n+=BigInt(1);",
        "readonly let q:=Deque([1]);q.push_back(2);",
        "let a:=[1];let view:=readonly_slice(a);view[0]=2;",
        "let a:=[1];let view:=readonly_slice(a);view[0]:=2;",
        "let a:=Array::<Int,2>([1,2]);let b:Array<Int,3>:=a;",
    ] {
        let out = Case::new(source).compile();
        assert!(!out.status.success(), "{source}");
        assert!(
            !String::from_utf8_lossy(&out.stderr).contains("internal compiler error"),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

#[test]
fn mutation_during_lazy_iteration_and_callbacks_is_detected() {
    for (source, message) in [
        (
            "let a:=IntArray([1,2]);let it:=a.map((x:Int)=>{a.push_back(3);x});__iter_next__(it);",
            "container changed during iteration",
        ),
        (
            "let a:=Deque([1,2]);let it:=__iter__(a);a.push_front(3);__iter_next__(it);",
            "container changed during iteration",
        ),
        (
            "let a:=Array::<Int,2>([1,2]);let it:=__iter__(a);a=Array::<Int,2>([3,4]);__iter_next__(it);",
            "container changed during iteration",
        ),
        (
            "let a:=[1,2];next_permutation_by_comp(a,(x:Int,y:Int)=>{a.push_back(3);x<y});",
            "assertion failed",
        ),
        (
            "let a:=[1,2];partition(a,(x:Int)=>{a.push_back(3);true});",
            "container changed during callback",
        ),
    ] {
        let out = Case::new(source).run("");
        assert_eq!(
            out.status.code(),
            Some(1),
            "{source}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(
            String::from_utf8_lossy(&out.stderr).contains(message),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

#[test]
fn standard_library_example() {
    let out = Case::new(include_str!("../examples/beta11/standard_library.rusk")).run("");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(out.stdout, b"1 1 2\n1 2 1\n2 1 1\n2 1 1\n1234567890123456789012345678900 -3 2\n1 2 4 7\n01010 3\n1 2 3 3\n1 8 3 8 3\n1 3 3 2\n1 2 a b\n6 72 1 3 6\n");
}

#[test]
fn ordered_comparators_cannot_reenter_native_trees() {
    let source = include_str!("../examples/beta11/ordered_reentry.rusk");
    for constructor in [
        "OrderedSet::<Key>()",
        "MultiSet::<Key>()",
        "OrderedDict::<Key,Int>()",
    ] {
        let code = source.replace("OrderedSet::<Key>()", constructor);
        let code = if constructor.contains("Dict") {
            code.replace("table.insert(", "table.set(").replace(
                "value: 1\n        }\n    );",
                "value: 1\n        }, 3\n    );",
            )
        } else {
            code
        };
        let out = Case::new(&code).run("");
        assert_eq!(
            out.status.code(),
            Some(1),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(out.stdout, b"before\n");
        assert!(
            String::from_utf8_lossy(&out.stderr)
                .contains("container reentered during ordered operation"),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

#[test]
fn packed_bit_operations_cross_word_boundaries_and_mask_the_tail() {
    let mut source = String::new();
    let mut expected = String::new();
    for n in [0usize, 1, 63, 64, 65, 127, 128] {
        let mask = if n == 128 {
            u128::MAX
        } else {
            (1u128 << n) - 1
        };
        let a = 0xa5a5_0123_fedc_4567_89ab_cdef_1357_2468u128 & mask;
        let b = 0x3141_5926_5358_9793_2384_6264_3383_2795u128 & mask;
        let bits = |x: u128| {
            if n == 0 {
                String::new()
            } else {
                format!("{x:0n$b}")
            }
        };
        source.push_str(&format!("{{let a:=BitVec(\"{}\");let b:=BitVec(\"{}\");wt Str(a&b),Str(a|b),Str(a^b),Str(~a),a.count();",bits(a),bits(b)));
        expected.push_str(&format!(
            "{} {} {} {} {}\n",
            bits(a & b),
            bits(a | b),
            bits(a ^ b),
            bits(!a & mask),
            a.count_ones()
        ));
        for shift in [-129i64, -65, -64, -1, 0, 1, 63, 64, 65, 129] {
            let left = if shift.unsigned_abs() >= 128 {
                0
            } else if shift < 0 {
                a >> (-shift as u32)
            } else {
                (a << (shift as u32)) & mask
            };
            let right = if shift.unsigned_abs() >= 128 {
                0
            } else if shift < 0 {
                (a << (-shift as u32)) & mask
            } else {
                a >> (shift as u32)
            };
            source.push_str(&format!("wt Str(a<<({shift})),Str(a>>({shift}));"));
            expected.push_str(&format!("{} {}\n", bits(left), bits(right)));
        }
        source.push('}');
    }
    let out = Case::new(&source).run("");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(out.stdout, expected.as_bytes());
}

#[test]
fn bigint_and_packed_integer_input_use_standard_io() {
    let source = "in a:BigInt;in b(3):IntArray;wt a,b;wt a+BigInt(b.sum());";
    let case = Case::new(source);
    for options in [vec![], vec!["--no-opt"]] {
        let out = case.run_with_options("-123456789012345678901234567890 4 -5 6", &options);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(
            out.stdout,
            b"-123456789012345678901234567890 4 -5 6\n-123456789012345678901234567885\n"
        );
    }
}

#[test]
fn rollback_union_find_restores_sizes_and_branches() {
    let source = r#"
let uf := RollbackUnionFind(5);
let start := uf.snapshot();
wt uf.unite(0, 1), uf.unite(1, 0), uf.snapshot(), uf.size(0);
let branch := uf.snapshot();
uf.unite(2, 3);
uf.unite(1, 2);
let detached = uf;
uf.rollback(branch);
wt uf.same(0, 3), uf.size(3), detached.size(3);
uf.unite(0, 4);
wt uf.same(1, 4), uf.size(4);
uf.rollback(start);
wt uf.size(0), uf.same(0, 4), uf.snapshot(), RollbackUnionFind(0).snapshot();
"#;
    for options in [vec![], vec!["--no-opt"]] {
        let out = Case::new(source).run_with_options("", &options);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(out.stdout, b"Yes No 1 2\nNo 1 4\nYes 3\n1 No 0 0\n");
    }
}

#[test]
fn number_theory_combines_congruences_and_handles_int_boundaries() {
    let source = r#"
let smallest := -9223372036854775807 - 1;
let (g,x,y) := extended_gcd(smallest, 0);
wt g, Int128(smallest)*x, y;
let (d,u,v) := extended_gcd(-15, 21);
wt d, -15i128*u+21i128*v;
let (zero,_,_) := extended_gcd(0,0);
wt zero;
let compatible := crt([(-2,6),(4,8),(0,1)]);
let (r,m) := compatible.value();
wt r,m,crt([(3,4),(4,6)]).is_none();
let empty: Vec<(Int,Int)> := [];
let (r0,m0) := crt(empty).value();
wt r0,m0;
wt isqrt(0),isqrt(1),isqrt(15),isqrt(16),isqrt(9223372036854775807);
"#;
    for options in [vec![], vec!["--no-opt"]] {
        let out = Case::new(source).run_with_options("", &options);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(
            out.stdout,
            b"9223372036854775808 9223372036854775808 0\n3 3\n0\n4 24 Yes\n0 1\n0 1 3 4 3037000499\n"
        );
    }
}

#[test]
fn persistent_segment_tree_preserves_versions_and_value_order() {
    let source = r#"
fn concatenate(a:Str,b:Str)->Str { return a+b; }
let values := ["a","b","c","d","e"];
let base := PersistentSegTree(values, "", concatenate);
values[1]="!";
let first := base.updated(1,"X");
let second := first.updated(3,"Y");
let branch := base.updated(4,"Z");
let handle = second;
let result := handle.get(3);
result="!";
wt base.prod(0,5), first.prod(0,5), second.prod(0,5), branch.prod(0,5), handle.get(3);
wt base.prod(1,4), base.prod(3,3)=="";
let no_values:Vec<Str> := [];
wt PersistentSegTree(no_values,"",concatenate).prod(0,0)=="";
"#;
    for options in [vec![], vec!["--no-opt"]] {
        let out = Case::new(source).run_with_options("", &options);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(out.stdout, b"abcde aXcde aXcYe abcdZ Y\nbcd Yes\nYes\n");
    }
}

#[test]
fn graph_algorithms_cover_disconnected_vertices_cycles_and_wide_weights() {
    let source = r#"
let maximum := 9223372036854775807;
let distance := dijkstra(5, [(0,1,maximum),(1,2,maximum),(0,3,10),(0,3,0),(3,3,0)], 0);
wt distance[0].value(), distance[1].value(), distance[2].value(), distance[3].value(), distance[4].is_none();
let order := topological_sort(5,[(0,2),(0,2),(1,2),(2,3)]).value();
wt order[0],order[1],order[2],order[3],order[4];
wt topological_sort(1,[(0,0)]).is_none(),topological_sort(0,[]).value().len();
let scc := strongly_connected_components(5,[(0,1),(1,0),(1,2),(2,3),(3,2),(3,4)]);
wt scc.len(),scc[0][0],scc[0][1],scc[1][0],scc[1][1],scc[2][0];
wt strongly_connected_components(0,[]).len();
let (weight, used) := minimum_spanning_forest(5,[(0,0,-9),(0,1,-2),(0,1,-2),(1,2,5),(0,2,7),(3,4,-1)]);
wt weight,used[0],used[1],used[2];
let (zero,nothing) := minimum_spanning_forest(0,[]);
wt zero,nothing.len();
"#;
    for options in [vec![], vec!["--no-opt"]] {
        let out = Case::new(source).run_with_options("", &options);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(
            out.stdout,
            b"0 9223372036854775807 18446744073709551614 0 Yes\n0 1 4 2 3\nYes 0\n3 0 1 2 3 4\n0\n2 1 5 3\n0 0\n"
        );
    }
}

#[test]
fn suffix_arrays_sort_strings_and_integer_sequences_lexicographically() {
    let source = r#"
let sa := suffix_array("banana");
let lcp := lcp_array("banana", sa);
wt sa[0],sa[1],sa[2],sa[3],sa[4],sa[5];
wt lcp[0],lcp[1],lcp[2],lcp[3],lcp[4];
let ints := [-9, 9223372036854775807, -9, -9];
let order := suffix_array(ints);
let lengths := lcp_array(ints, order);
wt order[0],order[1],order[2],order[3],lengths[0],lengths[1],lengths[2];
wt suffix_array("").len(), lcp_array("", []).len(), lcp_array("x", [0]).len();
"#;
    for options in [vec![], vec!["--no-opt"]] {
        let out = Case::new(source).run_with_options("", &options);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(
            out.stdout,
            b"5 3 1 0 4 2\n1 3 0 0 2\n3 2 0 1 1 1 0\n0 0 0\n"
        );
    }
}

#[test]
fn beta12_standard_library_example() {
    let source = include_str!("../examples/beta12/standard_library.rusk");
    for options in [vec![], vec!["--no-opt"]] {
        let out = Case::new(source).run_with_options("", &options);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(
            out.stdout,
            b"40 29 1\n2 No 3\n10 28 37\n7 Yes\n0 1 2\n2\n7 0 1\n5 3 1 0 4 2\n1 3 0 0 2\n3 3 4 24 3\n"
        );
    }
}

#[test]
fn beta12_rejects_invalid_inputs() {
    for (source, diagnostic) in [
        (
            "let uf := RollbackUnionFind(0); uf.rollback(1);",
            "assertion failed",
        ),
        (
            "let uf := RollbackUnionFind(0); wt uf.size(0);",
            "assertion failed",
        ),
        (
            "let t := PersistentSegTree([1], 0, (a: Int, b: Int) => a + b); t.updated(1, 0);",
            "assertion failed",
        ),
        ("dijkstra(2,[(1,1,-1)],0);", "assertion failed"),
        ("topological_sort(2,[(0,2)]);", "assertion failed"),
        ("lcp_array(\"aba\", [0,0,1]);", "invalid suffix array index"),
        ("lcp_array(\"aba\", [0,1,2]);", "suffix array is not sorted"),
        ("crt([(0,0)]);", "assertion failed"),
        ("crt([(0,9223372036854775807),(0,2)]);", "integer overflow"),
        ("isqrt(-1);", "assertion failed"),
    ] {
        let out = Case::new(source).run("");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert_eq!(out.status.code(), Some(1), "{source}: {stderr}");
        assert!(stderr.contains(diagnostic), "{source}: {stderr}");
    }
}
