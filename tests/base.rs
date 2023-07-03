use substruct::{substruct_child, substruct_use, SubstructRoot};

#[derive(SubstructRoot)]
struct Query {
    name: String,
    age: i32,
}

#[substruct_child(root = Query, fields(name))]
struct QueryA;

#[substruct_use(root = Query, fields(name))]
fn get_name(query: _) -> String {
    query.name().clone()
}

#[test]
fn test_that_it_works() {
    let query = Query {
        name: "John".to_string(),
        age: 32,
    };

    assert_eq!(query.name(), "John");
    assert_eq!(query.age(), &32);

    assert_eq!(get_name(query), "John");

    let query_a = QueryA {
        name: "Jack".to_string(),
    };
    assert_eq!(query_a.name(), "Jack");
    assert_eq!(get_name(query_a), "Jack");
}
