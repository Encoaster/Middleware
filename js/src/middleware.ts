
import { RpcWebSocketClient as RpcClient } from 'rpc-websocket-client';

type EncodeCallback = (data: object) => void;

class Middleware {
	private rpc: RpcWebSocketClient;
	private url: string;
	private token;
	constructor(url: string) {
		this.rpc = new RpcClient();
		this.url = url;
	}

	/**
	 * Authenticate the client with the server.
	 */
	login(user: string, pass: string): Promise<unknown> {
		return this.rpc.connect(this.url)
		.then(() => this.rpc.call('auth.login', {user: user, pass: pass}))
		.then((token) => this.token = token);
	}

	/**
	 * Encode a video, using callback to keep updates on progress.
	 */
	encode(callback: EncodeCallback, file: string, opt: any[string] = []): Promise<unknown> {
		return this.rpc.call('encode.start', {token: this.token, file: file, opt: opt})
		.then((id) => {
			this.rpc.onNotification.push((data) => {
				if (data.method !== 'encode') return;
				const params = data.params;
				if (id !== params.id) return;
				callback(params);
			});
		});
	}
}
