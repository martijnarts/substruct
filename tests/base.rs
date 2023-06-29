use smooth_grahpql::{Smooth, smooth_use};

#[derive(Smooth)]
struct Query {
    name: String,
    age: i32,
}

#[smooth_use(root = Query, fields(name))]
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
}
