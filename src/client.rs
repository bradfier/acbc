use crate::protocol::inbound::{InboundMessage, RealtimeUpdate, RealtimeCarUpdate, EntrylistUpdate, EntrylistCar, TrackData, BroadcastingEvent};
use crate::protocol::outbound::{OutboundMessage, RegistrationRequest, UnregisterRequest};
use log::{info, trace, debug, warn};
use nom_supreme::error::ErrorTree;
use nom_supreme::final_parser::{ByteOffset, Location};
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::time::Duration;
use thiserror::Error;

pub trait MessageHandler {
    fn realtime_update(&self, update: &RealtimeUpdate) {
        trace!("Received realtime update packet for time {}", update.session_time);
    }

    fn realtime_car_update(&self, update: &RealtimeCarUpdate) {
        trace!("Received realtime car update for car ID {}", update.id);
    }

    fn entrylist_update(&self, update: &EntrylistUpdate) {
        debug!("Received entry list update with {} cars", update.car_ids.len());
    }

    fn entrylist_car(&self, car: &EntrylistCar) {
        debug!("Received car information packet for car ID {}", car.id);
    }

    fn track_data(&self, track_data: &TrackData) {
        debug!("Received track data packet for {}", track_data.name);
    }

    fn broadcasting_event(&self, event: &BroadcastingEvent) {
        debug!("Received broadcasting event {:?}", event.event_type);
    }

}

pub struct BroadcastingClient<H: MessageHandler> {
    connection_id: u32,
    remote_addr: SocketAddr,
    listening_socket: UdpSocket,
    handler: H,
}

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("Server returned registration error: {0}")]
    RegistrationError(String),
    #[error("Failed decoding packet at location {0:?}")]
    MessageDecodeError(ErrorTree<ByteOffset>),
    #[error("Socket error: {0}")]
    SocketError(#[from] std::io::Error),
}

impl<H> BroadcastingClient<H>
where
    H: MessageHandler,
{
    pub fn connect<A: ToSocketAddrs>(
        listen: A,
        remote: A,
        handler: H,
        req: RegistrationRequest,
    ) -> Result<Self, ClientError> {
        // Bind the listening socket first
        let socket = UdpSocket::bind(listen)?;
        let remote_addr = remote.to_socket_addrs()?.next().unwrap();
        socket.connect(remote_addr.clone())?;

        // Then transmit the registration request
        let mut buffer = vec![];
        req.encode(&mut buffer)?;
        socket.send(&buffer)?;

        // Set a 5s timeout and wait for the registration reply to come back, if we get something else, panic for now
        socket.set_read_timeout(Some(Duration::from_secs(5)))?;
        let mut incoming = [0u8; 65535];
        let size = socket.recv(&mut incoming)?;

        let packet = &incoming[..size];

        let connection_id: u32 = match InboundMessage::decode(&packet) {
            Ok(InboundMessage::RegistrationResult(res)) => {
                if res.connection_success {
                    info!("Successfully registered with ACC Server");
                    Ok(res.connection_id)
                } else {
                    Err(ClientError::RegistrationError(
                        res.error_message.to_string(),
                    ))
                }
            }
            Ok(msg) => {
                // Without the connection ID we can't shut down later on, so we cannot continue
                panic!("Recevied a non-registration reply as first incoming packet!");
            }
            Err(e) => Err(ClientError::MessageDecodeError(e)),
        }?;

        // Put the socket back in blocking mode
        socket.set_read_timeout(None)?;

        Ok(Self {
            connection_id,
            remote_addr: remote_addr,
            listening_socket: socket,
            handler,
        })
    }

    pub fn shutdown(self) -> Result<(), std::io::Error> {
        let unregister = UnregisterRequest::new(self.connection_id);
        let mut buffer = vec![];
        unregister.encode(&mut buffer)?;

        self.listening_socket.send(&buffer)?;
        Ok(())
    }

    pub fn poll(&self) -> Result<(), ClientError> {
        let mut buffer = [0u8; 65535];
        let size = self.listening_socket.recv(&mut buffer)?;
        let decoded = InboundMessage::decode(&buffer[..size]).map_err(ClientError::MessageDecodeError)?;

        match decoded {
            InboundMessage::RealtimeUpdate(rt) => self.handler.realtime_update(&rt),
            InboundMessage::RealtimeCarUpdate(rt) => self.handler.realtime_car_update(&rt),
            InboundMessage::EntrylistUpdate(list) => self.handler.entrylist_update(&list),
            InboundMessage::EntrylistCar(car) => self.handler.entrylist_car(&car),
            InboundMessage::TrackData(track) => self.handler.track_data(&track),
            InboundMessage::BroadcastingEvent(event) => self.handler.broadcasting_event(&event),
            InboundMessage::RegistrationResult(_) => (),
        }
        Ok(())
    }
}
