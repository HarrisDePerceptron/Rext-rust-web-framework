
## Rext: Rust Axum Web Framework

Rext is an opinionated framework design to follow best practices for designing and writing backend business logic in Rust :crab:

### Features
- File structure and organization
- Tokio as the async provider
- Axum as the api web framework
- Ready to use WebSockets
- Websockets design to work with rooms (join, leave, message)
- Mongo data access layer included
- Redis Adapter for working with redis async commands
- Stateful horizontal scaling of Websockets

### Getting Started

Fire away 🚀

```
$ cargo run
```

### Writing Apis

**Model**

```

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    email: String,
    password: String,
}

impl User {
// model methods
}

```

**Data Access layer**


`app/user/user_dao.rs`
```
pub struct UserDao {
    fac: Arc<ApplicationFactory>,
    collection_name: String,
}

impl DaoObj<User> for UserDao {
fn get_factory(&self) -> Arc<ApplicationFactory> {
        self.fac.clone()
    }

fn get_collection_name(&self) -> &str {
        &self.collection_name
    }
}
```

**Service**

`app/user/user_service.rs`
```
pub struct UserService {
    dao: Arc<UserDao>,
}

impl Service<User> for UserService {
    fn get_dao(&self) -> Arc<dyn DaoObj<User>> {
        self.dao.clone()
    }
}

impl UserService {
// extending service methods
}


```

**Routes**

`app/user/user_routes.rs`


```
pub fn user_routes() -> Router<Arc<ServerState>> {
    Router::new()
        .route("/", post(user_create))
        .route("/", get(list_user))
        .route("/:user_id", get(get_user))
        .route("/users/login", post(user_login))
}
```


**Registering Routes**

`server.rs`

```
pub async fn server(...) -> ... {
// code 

let app = Router::new()
    ...
    .nest("/user", user::user_routes::user_routes())
    ...;
}
```

### WebSockets

Websockets are setup-ed out of the box. Extend websocket functionality by extending the commands

`websocket/messages.rs`
```
pub struct RoomMessage {
    pub message: String,
    pub room: String,
}

pub enum Command {
    ...
    NewCommand(String)
}

```
```
pub async fn parse_text_messages(...) -> ... {
    // code
    match command {
        ...,
        Command::NewCommand(v) => {

        },
        ...
    }

```

**Authentication**

Websocket auth is implemented through the authorization header 
which accepts a bearer token and decodes the token for user id represented by the `sub` payload field



### Secrets
Add your secrets to the  `secrets.rs`

```
pub static REDIS_URI: Lazy<String> =
    Lazy::new(|| env::var("REDIS_URI").expect("REDIS_URI not found"));

```


