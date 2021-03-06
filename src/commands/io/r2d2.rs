use reql_io::r2d2;
use errors::Error;
use super::io_error;
use {ConnectionManager, Codec, Connection, Result};
use reql_io::futures::sync::oneshot;
use reql_io::tokio_core::net::TcpStream;
use reql_io::tokio_core::io::Io;
use reql_io::futures::{Future, Stream};

impl r2d2::ManageConnection for ConnectionManager {
    type Connection = Connection;
    type Error = Error;

    fn connect(&self) -> Result<Connection> {
        Connection::new(self.clone())
    }

    //fn is_valid(&self, mut conn: &mut Connection) -> Result<()> {
    fn is_valid(&self, _: &mut Connection) -> Result<()> {
        unimplemented!();
    }

    fn has_broken(&self, conn: &mut Connection) -> bool {
        conn.broken
    }
}

impl Connection {
    fn new(manager: ConnectionManager) -> Result<Connection> {
        let logger = manager.logger;
        let remote = manager.remote;
        let server = manager.server;
        let addresses = server.0.clone();
        let (tx, rx) = oneshot::channel();

        remote.spawn(move |handle| {
            for address in addresses {
                if let Ok(stream) = TcpStream::connect(&address, handle).wait() {
                    tx.complete(stream);
                    return Ok(());
                }
            }
            Err(())
        });

        let stream = match rx.wait() {
            Ok(res) => res,
            Err(err) => {
                return Err(From::from(io_error(err)));
            }
        };

        let logger = logger.new(o!(
                "local_addr" => stream.local_addr()?.to_string(),
                "peer_addr" => stream.peer_addr()?.to_string(),
        ));

        /*
           let mut version = [0; 4];
           LittleEndian::write_u32(&mut version, V1_0 as u32);

           let handshake = transport
        // Send desired version to the server
        .send(version.as_ref().to_owned())

        // Send client first message
        .and_then(|transport| {
        let scram = try!(ClientFirst::new(cluster.user, cluster.password, None));
        let (scram, client_first) = scram.client_first();

        let ar = AuthRequest {
        protocol_version: 0,
        authentication_method: String::from("SCRAM-SHA-256"),
        authentication: client_first,
        };
        let mut msg = try!(to_vec(&ar));
        msg.push(b'\0');

        transport.send(version.as_ref().to_owned())
        })

        .and_then(|transport| transport.into_future().map_err(|(e, _)| e))
        .and_then(|(res, transport)| {
        match res {
        Some(ref msg) => {
        Ok(transport)
        }
        _ => {
        Err(io_error("invalid handshake"))
        }
        }
        })
        ;
        */

        let transport = stream.framed(Codec);

        Ok(Connection {
            id: 0,
            broken: false,
            server: server,
            transport: transport,
            logger: logger,
        })
    }
}
