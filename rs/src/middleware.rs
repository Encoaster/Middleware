
use jsonrpc_pubsub::{PubSubHandler, Session, Subscriber, SubscriptionId, Sink};
use jsonrpc_ws_server::{ServerBuilder, RequestContext};
use jsonrpc_core::*;

use std::collections::HashMap;
use std::sync::{Arc, RwLock, Mutex};

use serde_json::Map;
use serde_derive::Deserialize;

/// Keep track of user
pub trait Rpc {
	fn new() -> Self;
	/// A new user has logged in
	fn auth_login(&self, user: String, pass: String) -> Result<Value>;
	/// Start an encoding
	fn encode(&self, token: Value, encoding: Encoding) -> Result<()>;
}

#[derive(Deserialize)]
struct ParamsAuthLogin {
	user: String,
	pass: String,
}

#[derive(Deserialize)]
struct ParamsEncode {
	token: Value,
	file: String,
	opt: Value,
}

pub struct Encoding {
	pub file: String,
	pub opt: Value, // Note: Could be put in individual parameters
	sink: Sink,
}
impl Encoding {
	pub fn new(file: String, opt: Value, sink: Sink) -> Encoding {
		Encoding {
			file: file,
			opt: opt,
			sink: sink,
		}
	}
	pub fn update(&self, data: Map<String, Value>) -> bool {
		match self.sink.notify(Params::Map(data)) {
			Ok(_) => true,
			Err(_) => false,
		}
	}
}

pub struct Middleware<R: Send + Sync> {
	encodings: Arc<RwLock<HashMap<SubscriptionId, Encoding>>>,
	r: Arc<Mutex<R>>,
}
impl<R: Rpc + Send + Sync> Middleware<R> {
	pub fn new() -> Self {
		Self {
			encodings: Arc::default(),
			r: Arc::new(Mutex::new(R::new())),
		}
	}
	pub fn start(&mut self) {
		let mut io = PubSubHandler::new(MetaIoHandler::default());
		io.add_sync_method("auth.login", |params: Params| {
			let parsed: ParamsAuthLogin = params.parse().unwrap();
			self.r.lock().unwrap().auth_login(parsed.user, parsed.pass)
		});
		io.add_subscription(
			"encode",
			("encode.start", |params: Params, _meta, subscriber: Subscriber| {
				if params == Params::None {
					subscriber.reject(Error {
						code: ErrorCode::ParseError,
						message: "Invalid parameters.".into(),
						data: None,
					}).unwrap();
					return;
				}
				let parsed: ParamsEncode = params.parse().unwrap();
				println!("Start encoding");

				// TODO: Generate semi-random id (uuid.v1?).
				// jsonrpc has some default function for that.
				let sub_id = SubscriptionId::Number(1);
				let sink = subscriber.assign_id(sub_id).unwrap();
				let encoding = Encoding::new(parsed.file, parsed.opt, sink);

				match self.r.lock().unwrap().encode(parsed.token, encoding) {
					Ok(_) => {
						self.encodings.write().unwrap().insert(sub_id, encoding);
					},
					Err(_) => {}
				}
			}),
			("encode.abort", |id: SubscriptionId, _meta| -> BoxFuture<Result<Value>> {
				println!("Aborting encoding");
				let removed = self.encodings.write().unwrap().remove(&id);
				Box::pin(futures::future::ok(Value::Bool(removed.is_some())))
			}),
		);

		// Start the server with a websocket.
		let server = ServerBuilder::with_meta_extractor(io, |context: &RequestContext| {
			Arc::new(Session::new(context.sender()))
		})
			.start(&"0.0.0.0:8080".parse().unwrap())
			.expect("Unable to start RPC service");

		server.wait().unwrap();
	}
}

struct TestRpc {
}

impl Rpc for TestRpc {
	fn new() -> Self {
		Self {}
	}
	fn auth_login(&self, user: String, pass: String) -> Result<Value> {
		Ok(Value::String("ok".into()))
	}
	fn encode(&self, token: Value, encoding: Encoding) -> Result<()> {
		Ok(())
	}
}

fn main() {
	// Note: Temporary, may need to change in order to work properly.
	let mut mid: Middleware<TestRpc> = Middleware::new();
	mid.start();
}
