# Smooth GraphQL

The goal here is to create a smoother GraphQL client experience in Rust.

## Problem

In most GraphQL APIs, every field is optional. Secondly, the query defines the
structure of the outcome, which means that some functions will have more or 
fewer fields to access.

However, we still want to write generic functions over combinations of those 
fields. For example, imagine a GraphQL API:

```graphql
scalar Date

type User {
    id: ID
    name: String
    createdAt: Date
}
```

Function A querying id, name, and created_at and Function B querying only 
id and created_at would have different Rust structs generated for them since 
there are different fields returned:

```rust
struct FunctionAReturn {
    id: String,
    name: String,
    created_at: DateTime<Utc>,
}

struct FunctionBReturn {
    id: String,
    created_at: DateTime<Utc>,
}
```

A generic function over, for example, the id and created_at field cannot 
handle both of these structs:

```rust
fn print_age(values: _) { // What is values?
    println!("{}: {:?}", values.id, values.created_at)
}
```

## Proposal

I propose instead a set of proc_macros that add getter traits for each field,
and allow you to write generic functions easily over the sum of those traits:

```rust
#[derive(Smooth)]
struct User {
    id: String,
    name: String,
    created_at: DateTime<Utc>,
}

#[derive(SmoothChild)]
#[smooth_root(User)]
#[smooth_fields(id, name, created_at)]
struct FunctionAReturn; 

#[derive(SmoothChild)]
#[smooth_root(User)]
#[smooth_fields(id, created_at)]
struct FunctionBReturn; 

#[smooth_use(root = User, fields(id, created_at))]
fn get_name(query: _) {
    println!("{}: {:?}", query.id(), query.created_at())
}
```

This expands to something like:

```rust
struct User {
    id: String,
    name: String,
    created_at: DateTime<Utc>,
}
trait UserId {
    fn id(&self) -> String;
}
trait UserName {
    fn name(&self) -> String;
}
trait UserCreatedAt {
    fn created_at(&self) -> DateTime<Utc>;
}

impl UserId for User {
    fn id(&self) -> String {
        self.id
    }
}
impl UserName for User {
    fn name(&self) -> String {
        self.name
    }
}
impl UserCreatedAt for User {
    fn created_at(&self) -> String {
        self.created_at
    }
}

struct FunctionAReturn {
    id: String,
    name: String,
    created_at: DateTime<Utc>,
}
impl UserId for FunctionAReturn {
    fn id(&self) -> String {
        self.id
    }
}
impl UserName for FunctionAReturn {
    fn name(&self) -> String {
        self.name
    }
}
impl UserCreatedAt for FunctionAReturn {
    fn created_at(&self) -> String {
        self.created_at
    }
}

struct FunctionBReturn {
    id: String,
    created_at: DateTime<Utc>,
}
impl UserId for FunctionBReturn {
    fn id(&self) -> String {
        self.id
    }
}
impl UserCreatedAt for FunctionBReturn {
    fn created_at(&self) -> String {
        self.created_at
    }
}

trait GetNameInput: UserId + UserCreatedAt {}
impl<T: UserId + UserCreatedAt> GetNameInput for T {}
fn get_name(query: impl GetNameInput) {
    println!("{}: {:?}", query.id(), query.created_at())
}
```

# Status

This is incomplete, of course. I'm still working specifically on the 
`SmoothChild`, and the `smooth_use` is very unfinished. I'm sure there's 
other issues that will arrive over the course of finishing this.

Additionally, I've not tested integrating this with any existing GraphQL client 
implementations, nor have I thought about how to resolve the `Option<...>` 
nesting or nesting of GraphQL elements.

Who knows if I ever finish this :)
