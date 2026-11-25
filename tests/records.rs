mod support;
use support::Case;

fn succeeds(source: &str, expected: &str) {
    let out = Case::new(source).run("");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(out.stdout, expected.as_bytes());
}

#[test]
fn fields_share_update_and_rebind() {
    succeeds(
        r#"
struct Point { x: Int; y: Int; }
fn Point(x:Int,y:Int)->Point { return Point { y:y, x:x }; }
let x:=1; let point:=Point(x,2); let held:=point.x;
point.x=3; wt x,held,point.x;
point.x:=9; held=4; wt x,held,point.x;
let detached = point; detached.x=10; wt point.x,detached.x;
let alias:=point; let old:=point.x;
point=Point(20,30); wt alias.x,alias.y,old;
point=point; point.x=21; wt point.x,old;
"#,
        "3 3 3\n4 4 9\n9 10\n20 30 9\n21 9\n",
    );
}

#[test]
fn record_vectors_and_nested_copies_are_independent() {
    succeeds(
        r#"
struct Wrapper { points: Vec<Point>; }
struct Point { x: Int; }
let x:=1; let point:=Point { x:x };
let points:=[point,point]; let nested:=Wrapper { points:points };
let detached = nested; detached.points[0].x=7;
wt point.x,detached.points[0].x,detached.points[1].x;
let repeated:=[nested;2]; repeated[0].points[0].x=8;
wt repeated[0].points[0].x,repeated[0].points[1].x,repeated[1].points[0].x;
let saved:Point;
{ let temporary:=Point { x:9 }; saved:=temporary; }
let empty:Vec<Point>:=[]; empty.push_back(saved);
for p in empty { p.x+=1; }
wt empty[0].x;
let copied = empty; empty=[]; copied[0].x=11; wt saved.x,copied[0].x;
"#,
        "1 7 1\n8 1 1\n10\n10 11\n",
    );
}

#[test]
fn field_receivers_are_evaluated_once_before_rhs() {
    succeeds(
        r#"
struct Cell { value: Int; }
fn counted(count:Int,cell:Cell)->Cell { count+=1; return cell; }
let count:=0; let a:=Cell { value:1 }; let old:=a;
counted(count,a).value := (a:=Cell { value:2 }).value;
wt count,old.value,a.value;
old.value = (old.value := 3);
wt old.value;
let state:=0;
struct Pair { first:Int; second:Int; }
let pair:=Pair { second:(state+=1), first:copy(state) };
state=9; wt pair.first,pair.second,state;
"#,
        "1 2 2\n3\n1 9 9\n",
    );
}

#[test]
fn invalid_records_are_diagnosed_before_cpp() {
    for source in [
        "struct X { value:Int; value:Int; }",
        "struct X { value:Missing; }",
        "struct X { value:Int; } struct X { other:Int; }",
        "struct X { value:Int; } let x:=X { wrong:1 };",
        "struct X { a:Int; b:Int; } let x:=X { a:1 };",
        "struct X { value:Int; } let x:=X { value:true };",
        "struct X { value:Int; } let x:=X { value:1, value:2 };",
        "struct X { value:Int; } let x:=X { value:1 }; wt x.missing;",
        "struct X { value:Int; } let x:=X { value:1 }; x.value:=true;",
        "let x:=1; wt x.value;",
        "struct X { value:Int; } let x:=X();",
        "fn f() { struct X { value:Missing; } }",
        "struct X { value:Int; } let x:=[X { value:1 }]; x.sort();",
    ] {
        let case = Case::new(source);
        let out = case.compile();
        assert!(!out.status.success(), "{source}");
        assert!(
            String::from_utf8_lossy(&out.stderr).contains("main.rusk:"),
            "{source}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(!case.0.join("main.cpp").exists(), "{source}");
    }
}

#[test]
fn field_copying_dispatches_by_rusk_type_even_with_shared_cpp_types() {
    succeeds(
        r#"
extern symbol observed_copies { code: "int copied_fields=0;", }
extern type Score { cpp_name: "std::int64_t", }
extern fn score()->Score {
    cpp_name: "make_score",
    code: "Obj<std::int64_t> make_score(Loc) { return make_obj<std::int64_t>(7); }",
}
extern fn __copy__(value:Score)->Score {
    cpp_name: "copy_score",
    code: "Obj<std::int64_t> copy_score(Obj<std::int64_t> value, Loc) { ++copied_fields; return make_obj<std::int64_t>(*value); }",
}
extern fn copy_count()->Int {
    cpp_name: "count_score_copies",
    code: "Obj<std::int64_t> count_score_copies(Loc) { return make_obj<std::int64_t>(copied_fields); }",
}
struct Holder { value:Score; }
let original:=Holder { value:score() };
let detached = original;
detached=original;
let repeated:=[original;2];
let array:=[original]; let copied = array;
wt copy_count();
"#,
        "5\n",
    );
}

#[test]
fn empty_structs_support_tags_generic_identity_and_copying() {
    let source = r#"
struct Ascending {}
struct Descending {}
struct Marker<T> {}
fn direction(tag:Ascending)->Int{return 1;}
fn direction(tag:Descending)->Int{return -1;}
fn marker(tag:Marker<Int>)->Int{return 10;}
fn marker(tag:Marker<Str>)->Int{return 20;}
fn zero<T>()->T{return T();}
let made:Ascending:=zero();
let tag:=Ascending();let copied=tag;copied=tag;
let values:=[tag;2];let detached=values;
wt direction(copied),direction(detached[0]),direction(Descending());
wt marker(Marker::<Int>()),marker(Marker::<Str>());
let local:={struct Local{}let value:=Local();let clone=value;clone};
let clone=local;clone=local;
"#;
    let case = Case::new(source);
    for options in [&[][..], &["--no-opt"][..]] {
        let out = case.run_with_options("", options);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(out.stdout, b"1 1 -1\n10 20\n");
    }
    let case = Case::new("struct A{}struct B{}let a:=A();a:=B();");
    let out = case.compile();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("type mismatch"));
}

#[test]
fn manual_copy_hooks_override_automatic_copies_in_the_type_scope() {
    let source = r#"
struct Score { value:Int; }
fn __copy__(x:Score)->Score{return Score{value:x.value+10};}
let original:=Score{value:1};let detached=original;
wt detached.value,copy(original).value;
let repeated:=[original;2];let cloned= [original];
wt repeated[0].value,repeated[1].value,cloned[0].value;
let alias:=detached;detached=original;wt alias.value;
{
    struct Local { value:Int; }
    let offset:=20;
    fn __copy__(x:Local)->Local{return Local{value:x.value+offset};}
    let first:=Local{value:2};let second=first;
    let scalar=3;let xs=[1,2];
    wt second.value,copy(first).value,scalar,xs;
}
struct Box<T>{value:T;}
fn __copy__<T>(x:Box<T>)->Box<T>{return Box::<T>{value:x.value};}
let number:=5;let box:=Box::<Int>{value:number};let shared=box;
shared.value=7;wt number;
shared=box;wt box.value,shared.value;
enum Choice{Item(Int),Empty}
fn __copy__(x:Choice)->Choice{return Choice::Item(99);}
let choice=Choice::Empty;
wt match choice{Item(x)=>x,Empty=>0,};
"#;
    let case = Case::new(source);
    for options in [&[][..], &["--no-opt"][..]] {
        let out = case.run_with_options("", options);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(
            out.stdout,
            b"11 11\n11 11 11\n11\n22 22 3 1 2\n7\n7 7\n99\n"
        );
    }
}

#[test]
fn manual_copy_aliases_cannot_erase_readonly_or_noescape() {
    for (source, diagnostic) in [
        (
            "struct Item{value:Int;}fn __copy__(x:Item)->Item{return x;}readonly let a:=Item{value:1};let b=a;b.value=2;",
            "readonly",
        ),
        (
            "struct Item{value:Int;}fn __copy__(x:Item)->Item{return x;}fn leak(noescape x:Item)->Item{let y=x;return y;}leak(Item{value:1});",
            "noescape",
        ),
        (
            "struct Item{value:Int;}fn __copy__(x:Item)->Int{return 1;}let b=Item{value:2};",
            "type mismatch",
        ),
    ] {
        let case = Case::new(source);
        let out = case.compile();
        assert!(!out.status.success(), "{source}");
        assert!(
            String::from_utf8_lossy(&out.stderr).contains(diagnostic),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

#[test]
fn recursive_records_enums_and_containers_share_copy_and_break_cycles() {
    let source = include_str!("../examples/beta11/recursive_types.rusk");
    let case = Case::new(source);
    for options in [&[][..], &["--no-opt"][..]] {
        let out = case.run_with_options("", options);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(out.stdout, b"1\n2 9\n3\n");
    }
    let cpp = std::fs::read_to_string(case.0.join("main.cpp")).unwrap();
    let mut forwards = std::collections::BTreeSet::new();
    for line in cpp
        .lines()
        .filter(|line| line.starts_with("struct RuskRecord") && line.ends_with(';'))
    {
        assert!(
            forwards.insert(line),
            "duplicate forward declaration: {line}"
        );
    }
    let declarations = Case::new(
        "struct Direct{next:Direct;}enum Endless{Again(Endless)}{struct Local{next:Local;}}struct A{next:B;}struct B{next:A;}",
    );
    assert!(declarations.compile().status.success());
}

#[test]
fn automatic_equality_compares_record_fields_enum_tags_and_recursive_payloads() {
    let case = Case::new(
        r#"
struct Tag{}
struct Pair{first:Int;second:Str;}
wt Tag()==Tag(),Tag()!=Tag();
wt Pair{first:1,second:"a"}==Pair{first:1,second:"a"};
wt Pair{first:1,second:"a"}!=Pair{first:1,second:"b"};
enum Tree{Branch(Int,Vec<Tree>),Empty}
let first:=Tree::Branch(3,[Tree::Empty]);let cloned=first;
wt first==cloned,first!=Tree::Empty;
struct Local{value:Int;}
fn __eq__(x:Local,y:Local)->Bool{return x.value%10==y.value%10;}
wt Local{value:1}==Local{value:11},Local{value:1}!=Local{value:11};
"#,
    );
    for options in [&[][..], &["--no-opt"][..]] {
        let out = case.run_with_options("", options);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(out.stdout, b"Yes No\nYes\nYes\nYes Yes\nYes No\n");
    }
}

#[test]
fn structural_hashes_and_manual_policies_enable_user_defined_dictionary_keys() {
    let case = Case::new(
        r#"
struct Key{value:Int;label:Str;}
let original:=Key{value:1,label:"one"};
let entries:=Dict::<Key,Int>();entries.set(original,7);original.value=2;
wt entries.get(Key{value:1,label:"one"})?!,entries.contains(original);
let tuples:=Set::<(Key,Int)>();tuples.insert((Key{value:1,label:"a"},2));
wt tuples.contains((Key{value:1,label:"a"},2));
enum Kind{Value(Int),Other(Int),Empty}
let kinds:=Set::<Kind>();kinds.insert(Kind::Value(1));
wt kinds.contains(Kind::Value(1)),kinds.contains(Kind::Other(1));
wt hash(Kind::Value(1))==hash(Kind::Value(1));
struct Custom{value:Int;}
fn __eq__(a:Custom,b:Custom)->Bool{return a.value%10==b.value%10;}
fn __hash__(a:Custom)->Int{return hash(a.value%10);}
let custom:=Set::<Custom>();custom.insert(Custom{value:1});
wt custom.contains(Custom{value:11});
struct Empty{}
let empty:=Set::<Empty>();empty.insert(Empty());wt empty.contains(Empty());
"#,
    );
    for options in [&[][..], &["--no-opt"][..]] {
        let out = case.run_with_options("", options);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(out.stdout, b"7 No\nYes\nYes No\nYes\nYes\nYes\n");
    }
    let case = Case::new(
        "struct Key{x:Int;}fn __eq__(a:Key,b:Key)->Bool{return true;}let values:=Set::<Key>();",
    );
    let out = case.compile();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("__hash__"));
}

#[test]
fn expanding_recursive_record_specializations_fail_without_panicking() {
    let case = Case::new("struct Grow<T>{next:Grow<Vec<T>>;}let value:Grow<Int>;");
    let out = case.compile();
    let error = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(error.contains("limit"), "{error}");
    assert!(!error.contains("internal compiler error"), "{error}");
}

#[test]
fn manual_graph_copy_and_equality_handle_cycles_and_keep_copies_separate() {
    let case = Case::new(include_str!("../examples/beta11/graph_copy.rusk"));
    for options in [&[][..], &["--no-opt"][..]] {
        let out = case.run_with_options("", options);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(out.stdout, b"Yes\n1 10 No\n");
    }
}

#[test]
fn shared_manual_container_copies_preserve_assignment_sources_and_outer_identity() {
    let source = include_str!("../examples/beta11/shared_copy_assignment.rusk");
    for options in [vec![], vec!["--no-opt"]] {
        let out = support::Case::new(source).run_with_options("", &options);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(out.stdout, b"7 2 3 7 2 2\n2 1 8 1\n2 1 Yes\n");
    }
}

#[test]
fn shared_manual_container_copies_preserve_access_and_escape_restrictions() {
    for source in [
        "fn __copy__(v:Vec<Int>)->Vec<Int>{return v;}readonly let a:=[1];let b:=[0];b=a;b[0]=2;",
        "fn __copy__(v:Vec<Int>)->Vec<Int>{return v;}fn save(noescape a:Vec<Int>,b:Vec<Int>){b=a;}save([1],[0]);",
    ] {
        let out = Case::new(source).compile();
        assert!(!out.status.success(), "{source}");
        let error = String::from_utf8_lossy(&out.stderr);
        assert!(
            error.contains("readonly") || error.contains("noescape"),
            "{error}"
        );
    }
}
