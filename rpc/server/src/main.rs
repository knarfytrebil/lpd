extern crate grpc;
extern crate tls_api_native_tls;
extern crate tls_api;
extern crate httpbis;
extern crate ctrlc;
extern crate bitcoin;
extern crate secp256k1;
extern crate hex;
extern crate serde_json;


extern crate futures;

extern crate implementation;

extern crate connection;

extern crate wire;

mod config;
use self::config::{Argument, Error as CommandLineReadError};

mod wallet;

use grpc::Error as GrpcError;
use httpbis::Error as HttpbisError;
use std::io::Error as IoError;
use implementation::wallet_lib::error::WalletError;

use wire::MessageInfoJSON;
use futures::sync::mpsc::unbounded;

use std::io::Write;

#[derive(Debug)]
enum Error {
    Grpc(GrpcError),
    Httpbis(HttpbisError),
    CommandLineRead(CommandLineReadError),
    Io(IoError),
    WalletError(WalletError),
    SendError(ctrlc::Error),
    TransportError(connection::TransportError)
}

fn main() -> Result<(), Error> {
    use std::{sync::{Mutex, RwLock, Arc}, path::PathBuf};
    use grpc::ServerBuilder;
    use implementation::{Node, Command, routing_service, channel_service, payment_service, wallet_service};
    use futures::{sync::mpsc, Future, Sink};
    use self::Error::*;
    use self::wallet::create_wallet;

    let argument = Argument::from_env().map_err(CommandLineRead)?;

    let mut message_dump_file = std::fs::File::create("msg-dump.json")
        .map_err(|err| Io(err))?;

    let wallet = {
        let mut wallet_db_path = PathBuf::from(argument.db_path.clone());
        wallet_db_path.push("wallet");
        let wallet = create_wallet(&wallet_db_path.as_path()).map_err(WalletError)?;
        Arc::new(Mutex::new(wallet))
    };


    let (node, tx, rx) = {
        println!("the lightning peach node is listening rpc at: {}, listening peers at: {}, has database at: {}",
                 argument.address, argument.p2p_address, argument.db_path);

        let (tx, rx) = mpsc::channel(1);
        let tx_wait = tx.clone();
        ctrlc::set_handler(move || {
            println!("received termination command");
            tx_wait.clone().send(Command::Terminate).wait().unwrap();
            println!("the command is propagated, terminating...");
        }).map_err(SendError)?;

        // TODO: generate and store it somehow
        let secret = [
            0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12,
            0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12,
            0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12,
            0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12,
        ];

        let mut node_db_path = PathBuf::from(argument.db_path.clone());
        node_db_path.push("node");

        let (ch_message_dump_sender, ch_message_dump_receiver) = std::sync::mpsc::channel::<MessageInfoJSON>();

        // TODO(mkl): maybe future will suffice ?
        std::thread::spawn(move || {
            loop {
                // TODO(mkl): stop on node stop. If all peers and node are dropped then channel will close
                // Maybe we should done it earlier.
                match ch_message_dump_receiver.recv() {
                    Ok(msgInfo) => {
                        let s = match serde_json::to_string(&msgInfo) {
                            Ok(s) => s,
                            Err(err) => {
                                eprintln!("cannot serialize value to json: {:?}", err);
                                continue
                            }
                        };
                        match message_dump_file.write_all(&format!("{}\n", s).into_bytes()) {
                            Ok(_) => {},
                            Err(err) => {
                                eprintln!("cannot write to message dump file: {:?}", err);
                                return;
                            }
                        };
                    }
                    Err(err) => return,
                }
            }
        });

        (Arc::new(RwLock::new(Node::new(
            wallet.clone(),
            secret,
            node_db_path,
            ch_message_dump_sender,
        ))), tx, rx)
    };

    let server = {
        let mut server = ServerBuilder::new();
        if let Some(acceptor) = argument.tls_acceptor {
            server.http.set_tls(acceptor);
        }
        server.http.set_addr(argument.address).map_err(Httpbis)?;
        // TODO(mkl): make it configurable
        server.http.set_cpu_pool_threads(4);
        server.add_service(wallet_service(wallet.clone(), tx.clone()));
        server.add_service(routing_service(node.clone(), tx.clone()));
        server.add_service(channel_service());
        server.add_service(payment_service());
        server.build().map_err(Grpc)?
    };

    Node::listen(node, &argument.p2p_address, rx)
        .map_err(|err| {
            Error::TransportError(err)
        })?;

    // TODO: handle double ctrl+c
    //panic!("received termination command during processing termination command, terminate immediately");

    let _ = server;
    Ok(())
}
